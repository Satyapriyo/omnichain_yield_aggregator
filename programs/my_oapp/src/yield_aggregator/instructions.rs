use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, CloseAccount, SyncNative};
use anchor_spl::associated_token::AssociatedToken;
use crate::yield_aggregator::{state::*, events::*, errors::*};
use crate::{YIELD_AGGREGATOR_SEED, PROTOCOL_SEED, USER_POSITION_SEED, YIELD_VAULT_SEED};
use oapp::endpoint_cpi::{LzAccount, LzReceiveParams};

// ============================== Initialize Yield Aggregator ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeYieldAggregatorParams {
    pub admin: Pubkey,
}

#[derive(Accounts)]
pub struct InitializeYieldAggregator<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + YieldAggregator::INIT_SPACE,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl InitializeYieldAggregator<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &InitializeYieldAggregatorParams) -> Result<()> {
        let yield_aggregator = &mut ctx.accounts.yield_aggregator;
        yield_aggregator.admin = params.admin;
        yield_aggregator.total_protocols = 0;
        yield_aggregator.total_tvl = 0;
        yield_aggregator.total_yield_earned = 0;
        yield_aggregator.emergency_paused = false;
        yield_aggregator.bump = ctx.bumps.yield_aggregator;

        emit!(YieldAggregatorInitialized {
            admin: params.admin,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// ============================== Add Protocol ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AddProtocolParams {
    pub name: String,
    pub chain_id: u32,
    pub initial_apy: u64,
    pub max_capacity: u64,
    pub risk_score: u8,
}

#[derive(Accounts)]
#[instruction(params: AddProtocolParams)]
pub struct AddProtocol<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + ProtocolInfo::INIT_SPACE,
        seeds = [PROTOCOL_SEED, params.name.as_bytes()],
        bump
    )]
    pub protocol_info: Account<'info, ProtocolInfo>,
    #[account(
        mut,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump,
        has_one = admin @ YieldAggregatorError::Unauthorized
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl AddProtocol<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &AddProtocolParams) -> Result<()> {
        require!(params.risk_score >= 1 && params.risk_score <= 10, YieldAggregatorError::InvalidRiskScore);
        require!(params.name.len() <= 32, YieldAggregatorError::InvalidProtocolName);

        let protocol_info = &mut ctx.accounts.protocol_info;
        protocol_info.name = params.name.clone();
        protocol_info.chain_id = params.chain_id;
        protocol_info.current_apy = params.initial_apy;
        protocol_info.tvl = 0;
        protocol_info.max_capacity = params.max_capacity;
        protocol_info.risk_score = params.risk_score;
        protocol_info.is_active = true;
        protocol_info.last_update = Clock::get()?.unix_timestamp;
        protocol_info.bump = ctx.bumps.protocol_info;

        let yield_aggregator = &mut ctx.accounts.yield_aggregator;
        yield_aggregator.total_protocols += 1;

        emit!(ProtocolAdded {
            name: params.name,
            chain_id: params.chain_id,
            apy: params.initial_apy,
            max_capacity: params.max_capacity,
            risk_score: params.risk_score,
        });

        Ok(())
    }
}

// ============================== Deposit SOL for Yield ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DepositSolForYieldParams {
    pub amount: u64,
    pub target_protocol: String,
    pub target_chain_id: u32,
    pub min_apy: u64,
}

#[derive(Accounts)]
#[instruction(params: DepositSolForYieldParams)]
pub struct DepositSolForYield<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserPosition::INIT_SPACE,
        seeds = [USER_POSITION_SEED, user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = wsol_mint,
        associated_token::authority = user,
    )]
    pub user_wsol_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + YieldVault::INIT_SPACE,
        seeds = [YIELD_VAULT_SEED, wsol_mint.key().as_ref()],
        bump
    )]
    pub yield_vault: Account<'info, YieldVault>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = wsol_mint,
        associated_token::authority = yield_vault,
    )]
    pub vault_wsol_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [PROTOCOL_SEED, params.target_protocol.as_bytes()],
        bump = protocol_info.bump,
        constraint = protocol_info.is_active @ YieldAggregatorError::ProtocolInactive,
        constraint = protocol_info.current_apy >= params.min_apy @ YieldAggregatorError::ApyTooLow
    )]
    pub protocol_info: Account<'info, ProtocolInfo>,
    #[account(
        mut,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump,
        constraint = !yield_aggregator.emergency_paused @ YieldAggregatorError::EmergencyPaused
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    pub wsol_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl DepositSolForYield<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &DepositSolForYieldParams) -> Result<()> {
        require!(params.amount > 0, YieldAggregatorError::InvalidAmount);
        require!(
            ctx.accounts.user.lamports() >= params.amount + 10_000_000, // Keep 0.01 SOL for fees
            YieldAggregatorError::InsufficientBalance
        );

        // Step 1: Transfer SOL to user's WSOL account
        let transfer_sol_ix = anchor_lang::system_program::Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.user_wsol_account.to_account_info(),
        };
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                transfer_sol_ix,
            ),
            params.amount,
        )?;

        // Step 2: Sync WSOL account (SOL → WSOL conversion)
        let sync_native_ix = SyncNative {
            account: ctx.accounts.user_wsol_account.to_account_info(),
        };
        token::sync_native(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                sync_native_ix,
            ),
        )?;

        // Step 3: Transfer WSOL to yield vault
        let transfer_wsol_ix = Transfer {
            from: ctx.accounts.user_wsol_account.to_account_info(),
            to: ctx.accounts.vault_wsol_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                transfer_wsol_ix,
            ),
            params.amount,
        )?;

        // Step 4: Update user position
        let user_position = &mut ctx.accounts.user_position;
        user_position.user = ctx.accounts.user.key();
        user_position.total_deposits += params.amount;
        user_position.position_count += 1;
        user_position.last_activity = Clock::get()?.unix_timestamp;
        user_position.bump = ctx.bumps.user_position;

        // Step 5: Update yield vault
        let yield_vault = &mut ctx.accounts.yield_vault;
        yield_vault.mint = ctx.accounts.wsol_mint.key();
        yield_vault.authority = ctx.accounts.yield_vault.key();
        yield_vault.total_deposits += params.amount;
        yield_vault.bump = ctx.bumps.yield_vault;

        // Step 6: Update global aggregator
        let yield_aggregator = &mut ctx.accounts.yield_aggregator;
        yield_aggregator.total_tvl += params.amount;

        // Step 7: If cross-chain, emit request event
        if params.target_chain_id != 40168 { // 40168 is Solana's LayerZero endpoint ID
            emit!(CrossChainDepositRequested {
                user: ctx.accounts.user.key(),
                amount: params.amount,
                target_chain: params.target_chain_id,
                target_protocol: params.target_protocol.clone(),
                timestamp: Clock::get()?.unix_timestamp,
            });
        } else {
            emit!(LocalDepositProcessed {
                user: ctx.accounts.user.key(),
                amount: params.amount,
                protocol: params.target_protocol.clone(),
                apy: ctx.accounts.protocol_info.current_apy,
                timestamp: Clock::get()?.unix_timestamp,
            });
        }

        Ok(())
    }
}

// ============================== Deposit SPL Tokens for Yield ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DepositForYieldParams {
    pub amount: u64,
    pub target_protocol: String,
    pub target_chain_id: u32,
    pub min_apy: u64,
}

#[derive(Accounts)]
#[instruction(params: DepositForYieldParams)]
pub struct DepositForYield<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserPosition::INIT_SPACE,
        seeds = [USER_POSITION_SEED, user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + YieldVault::INIT_SPACE,
        seeds = [YIELD_VAULT_SEED, token_mint.key().as_ref()],
        bump
    )]
    pub yield_vault: Account<'info, YieldVault>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = yield_vault,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [PROTOCOL_SEED, params.target_protocol.as_bytes()],
        bump = protocol_info.bump,
        constraint = protocol_info.is_active @ YieldAggregatorError::ProtocolInactive,
        constraint = protocol_info.current_apy >= params.min_apy @ YieldAggregatorError::ApyTooLow
    )]
    pub protocol_info: Account<'info, ProtocolInfo>,
    #[account(
        mut,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump,
        constraint = !yield_aggregator.emergency_paused @ YieldAggregatorError::EmergencyPaused
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl DepositForYield<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &DepositForYieldParams) -> Result<()> {
        require!(params.amount > 0, YieldAggregatorError::InvalidAmount);
        require!(
            ctx.accounts.user_token_account.amount >= params.amount,
            YieldAggregatorError::InsufficientBalance
        );

        // Transfer tokens to yield vault
        let transfer_ix = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                transfer_ix,
            ),
            params.amount,
        )?;

        // Update user position
        let user_position = &mut ctx.accounts.user_position;
        user_position.user = ctx.accounts.user.key();
        user_position.total_deposits += params.amount;
        user_position.position_count += 1;
        user_position.last_activity = Clock::get()?.unix_timestamp;
        user_position.bump = ctx.bumps.user_position;

        // Update yield vault
        let yield_vault = &mut ctx.accounts.yield_vault;
        yield_vault.mint = ctx.accounts.token_mint.key();
        yield_vault.authority = ctx.accounts.yield_vault.key();
        yield_vault.total_deposits += params.amount;
        yield_vault.bump = ctx.bumps.yield_vault;

        // Update global aggregator
        let yield_aggregator = &mut ctx.accounts.yield_aggregator;
        yield_aggregator.total_tvl += params.amount;

        // Emit appropriate event
        if params.target_chain_id != 40168 {
            emit!(CrossChainDepositRequested {
                user: ctx.accounts.user.key(),
                amount: params.amount,
                target_chain: params.target_chain_id,
                target_protocol: params.target_protocol.clone(),
                timestamp: Clock::get()?.unix_timestamp,
            });
        } else {
            emit!(LocalDepositProcessed {
                user: ctx.accounts.user.key(),
                amount: params.amount,
                protocol: params.target_protocol.clone(),
                apy: ctx.accounts.protocol_info.current_apy,
                timestamp: Clock::get()?.unix_timestamp,
            });
        }

        Ok(())
    }
}

// ============================== Withdraw SOL ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawSolParams {
    pub amount: u64, // 0 means withdraw all
    pub as_native_sol: bool,
}

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    #[account(
        mut,
        seeds = [USER_POSITION_SEED, user.key().as_ref()],
        bump = user_position.bump,
        has_one = user @ YieldAggregatorError::Unauthorized
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(
        mut,
        associated_token::mint = wsol_mint,
        associated_token::authority = user,
    )]
    pub user_wsol_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [YIELD_VAULT_SEED, wsol_mint.key().as_ref()],
        bump = yield_vault.bump
    )]
    pub yield_vault: Account<'info, YieldVault>,
    #[account(
        mut,
        associated_token::mint = wsol_mint,
        associated_token::authority = yield_vault,
    )]
    pub vault_wsol_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump,
        constraint = !yield_aggregator.emergency_paused @ YieldAggregatorError::EmergencyPaused
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    pub wsol_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl WithdrawSol<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &WithdrawSolParams) -> Result<()> {
        let withdraw_amount = if params.amount == 0 {
            ctx.accounts.vault_wsol_account.amount
        } else {
            params.amount
        };

        require!(withdraw_amount > 0, YieldAggregatorError::InvalidAmount);
        require!(
            ctx.accounts.vault_wsol_account.amount >= withdraw_amount,
            YieldAggregatorError::InsufficientBalance
        );

        // Transfer WSOL from vault to user
        let vault_seeds = &[
            YIELD_VAULT_SEED,
            ctx.accounts.wsol_mint.key().as_ref(),
            &[ctx.accounts.yield_vault.bump],
        ];
        let signer_seeds = &[&vault_seeds[..]];

        let transfer_ix = Transfer {
            from: ctx.accounts.vault_wsol_account.to_account_info(),
            to: ctx.accounts.user_wsol_account.to_account_info(),
            authority: ctx.accounts.yield_vault.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer_ix,
                signer_seeds,
            ),
            withdraw_amount,
        )?;

        // If user wants native SOL, close the WSOL account
        if params.as_native_sol {
            let close_ix = CloseAccount {
                account: ctx.accounts.user_wsol_account.to_account_info(),
                destination: ctx.accounts.user.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            };
            token::close_account(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    close_ix,
                ),
            )?;
        }

        // Update user position
        let user_position = &mut ctx.accounts.user_position;
        user_position.total_deposits = user_position.total_deposits.saturating_sub(withdraw_amount);
        user_position.last_activity = Clock::get()?.unix_timestamp;

        // Update yield vault
        let yield_vault = &mut ctx.accounts.yield_vault;
        yield_vault.total_deposits = yield_vault.total_deposits.saturating_sub(withdraw_amount);

        // Update global aggregator
        let yield_aggregator = &mut ctx.accounts.yield_aggregator;
        yield_aggregator.total_tvl = yield_aggregator.total_tvl.saturating_sub(withdraw_amount);

        emit!(YieldWithdrawn {
            user: ctx.accounts.user.key(),
            amount: withdraw_amount,
            target_chain: 40168, // Solana
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// ============================== Withdraw Yield ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawYieldParams {
    pub amount: u64,
    pub target_chain_id: u32,
}

#[derive(Accounts)]
pub struct WithdrawYield<'info> {
    #[account(
        mut,
        seeds = [USER_POSITION_SEED, user.key().as_ref()],
        bump = user_position.bump,
        has_one = user @ YieldAggregatorError::Unauthorized
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(
        mut,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump,
        constraint = !yield_aggregator.emergency_paused @ YieldAggregatorError::EmergencyPaused
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl WithdrawYield<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &WithdrawYieldParams) -> Result<()> {
        require!(params.amount > 0, YieldAggregatorError::InvalidAmount);

        // Update user position
        let user_position = &mut ctx.accounts.user_position;
        user_position.last_activity = Clock::get()?.unix_timestamp;

        emit!(CrossChainWithdrawRequested {
            user: ctx.accounts.user.key(),
            amount: params.amount,
            target_chain: params.target_chain_id,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// ============================== Rebalance Position ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RebalancePositionParams {
    pub from_protocol: String,
    pub to_protocol: String,
    pub amount: u64,
    pub target_chain_id: u32,
}

#[derive(Accounts)]
#[instruction(params: RebalancePositionParams)]
pub struct RebalancePosition<'info> {
    #[account(
        mut,
        seeds = [USER_POSITION_SEED, user.key().as_ref()],
        bump = user_position.bump,
        has_one = user @ YieldAggregatorError::Unauthorized
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(
        seeds = [PROTOCOL_SEED, params.from_protocol.as_bytes()],
        bump = from_protocol.bump,
        constraint = from_protocol.is_active @ YieldAggregatorError::ProtocolInactive
    )]
    pub from_protocol: Account<'info, ProtocolInfo>,
    #[account(
        seeds = [PROTOCOL_SEED, params.to_protocol.as_bytes()],
        bump = to_protocol.bump,
        constraint = to_protocol.is_active @ YieldAggregatorError::ProtocolInactive
    )]
    pub to_protocol: Account<'info, ProtocolInfo>,
    #[account(
        mut,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump,
        constraint = !yield_aggregator.emergency_paused @ YieldAggregatorError::EmergencyPaused
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl RebalancePosition<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &RebalancePositionParams) -> Result<()> {
        require!(params.amount > 0, YieldAggregatorError::InvalidAmount);

        // Update user position
        let user_position = &mut ctx.accounts.user_position;
        user_position.last_activity = Clock::get()?.unix_timestamp;

        emit!(RebalanceRequested {
            user: ctx.accounts.user.key(),
            from_protocol: params.from_protocol.clone(),
            to_protocol: params.to_protocol.clone(),
            amount: params.amount,
            target_chain: params.target_chain_id,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// ============================== Update Yield Rates ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateYieldRatesParams {
    pub protocol_name: String,
    pub new_apy: u64,
}

#[derive(Accounts)]
#[instruction(params: UpdateYieldRatesParams)]
pub struct UpdateYieldRates<'info> {
    #[account(
        mut,
        seeds = [PROTOCOL_SEED, params.protocol_name.as_bytes()],
        bump = protocol_info.bump
    )]
    pub protocol_info: Account<'info, ProtocolInfo>,
    #[account(
        mut,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump,
        has_one = admin @ YieldAggregatorError::Unauthorized
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl UpdateYieldRates<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &UpdateYieldRatesParams) -> Result<()> {
        let protocol_info = &mut ctx.accounts.protocol_info;
        protocol_info.current_apy = params.new_apy;
        protocol_info.last_update = Clock::get()?.unix_timestamp;

        emit!(YieldRateUpdated {
            protocol: params.protocol_name,
            new_apy: params.new_apy,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// ============================== Compound Yield ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CompoundYieldParams {
    pub protocol_name: String,
}

#[derive(Accounts)]
#[instruction(params: CompoundYieldParams)]
pub struct CompoundYield<'info> {
    #[account(
        mut,
        seeds = [USER_POSITION_SEED, user.key().as_ref()],
        bump = user_position.bump,
        has_one = user @ YieldAggregatorError::Unauthorized
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(
        seeds = [PROTOCOL_SEED, params.protocol_name.as_bytes()],
        bump = protocol_info.bump,
        constraint = protocol_info.is_active @ YieldAggregatorError::ProtocolInactive
    )]
    pub protocol_info: Account<'info, ProtocolInfo>,
    #[account(
        mut,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump,
        constraint = !yield_aggregator.emergency_paused @ YieldAggregatorError::EmergencyPaused
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl CompoundYield<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &CompoundYieldParams) -> Result<()> {
        let user_position = &mut ctx.accounts.user_position;
        require!(user_position.total_yield_earned > 0, YieldAggregatorError::NoYieldToCompound);

        let yield_amount = user_position.total_yield_earned;
        user_position.total_deposits += yield_amount;
        user_position.total_yield_earned = 0;
        user_position.last_activity = Clock::get()?.unix_timestamp;

        emit!(YieldCompounded {
            user: ctx.accounts.user.key(),
            protocol: params.protocol_name.clone(),
            yield_amount,
            new_principal: user_position.total_deposits,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// ============================== Emergency Pause ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EmergencyPauseParams {
    pub pause: bool,
}

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(
        mut,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump,
        has_one = admin @ YieldAggregatorError::Unauthorized
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl EmergencyPause<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &EmergencyPauseParams) -> Result<()> {
        let yield_aggregator = &mut ctx.accounts.yield_aggregator;
        yield_aggregator.emergency_paused = params.pause;

        if params.pause {
            emit!(EmergencyPauseActivated {
                admin: ctx.accounts.admin.key(),
                timestamp: Clock::get()?.unix_timestamp,
            });
        } else {
            emit!(EmergencyPauseDeactivated {
                admin: ctx.accounts.admin.key(),
                timestamp: Clock::get()?.unix_timestamp,
            });
        }

        Ok(())
    }
}

// ============================== Get Optimal Strategy ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct GetOptimalStrategyParams {
    pub amount: u64,
    pub risk_tolerance: u8,
    pub min_apy: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct OptimalStrategyResponse {
    pub recommended_protocol: String,
    pub expected_apy: u64,
    pub risk_score: u8,
    pub chain_id: u32,
}

#[derive(Accounts)]
pub struct GetOptimalStrategy<'info> {
    #[account(
        seeds = [YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    pub user: Signer<'info>,
}

impl GetOptimalStrategy<'_> {
    pub fn apply(ctx: &Context<Self>, params: &GetOptimalStrategyParams) -> Result<OptimalStrategyResponse> {
        // This is a simplified implementation
        // In a real system, this would analyze all protocols and return the best match
        
        // For demo purposes, return a default strategy
        Ok(OptimalStrategyResponse {
            recommended_protocol: "marinade".to_string(),
            expected_apy: 850, // 8.5%
            risk_score: 6,
            chain_id: 40168, // Solana
        })
    }
}