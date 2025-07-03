use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, CloseAccount, SyncNative};
use anchor_spl::associated_token::AssociatedToken;
use crate::yield_aggregator::{state::*, events::*, errors::*};
use crate::{YIELD_AGGREGATOR_SEED, PROTOCOL_SEED, USER_POSITION_SEED, YIELD_VAULT_SEED};

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

// ============================== Deposit for Yield ==============================

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
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl DepositForYield<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &DepositForYieldParams) -> Result<()> {
        require!(params.amount > 0, YieldAggregatorError::InvalidAmount);

        let user_position = &mut ctx.accounts.user_position;
        user_position.user = ctx.accounts.user.key();
        user_position.total_deposits += params.amount;
        user_position.position_count += 1;
        user_position.last_activity = Clock::get()?.unix_timestamp;
        user_position.bump = ctx.bumps.user_position;

        let yield_aggregator = &mut ctx.accounts.yield_aggregator;
        yield_aggregator.total_tvl += params.amount;

        emit!(CrossChainDepositRequested {
            user: ctx.accounts.user.key(),
            amount: params.amount,
            target_chain: params.target_chain_id,
            target_protocol: params.target_protocol.clone(),
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
        // Simplified implementation - in production, this would analyze all protocols
        Ok(OptimalStrategyResponse {
            recommended_protocol: "marinade".to_string(),
            expected_apy: 850, // 8.5%
            risk_score: 6,
            chain_id: 40168, // Solana
        })
    }
}