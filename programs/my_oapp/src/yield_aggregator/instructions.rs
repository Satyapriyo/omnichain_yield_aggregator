use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::*};
use crate::{YIELD_AGGREGATOR_SEED, PROTOCOL_SEED, USER_POSITION_SEED};

// ============================== Initialize Yield Aggregator ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeYieldAggregatorParams {
    pub admin: Pubkey,
}

#[derive(Accounts)]
pub struct InitializeYieldAggregator<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + YieldAggregator::INIT_SPACE,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl InitializeYieldAggregator<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &InitializeYieldAggregatorParams) -> Result<()> {
        // Minimize stack usage by using references
        let aggregator = &mut ctx.accounts.yield_aggregator;
        let timestamp = Clock::get()?.unix_timestamp;
        
        // Initialize with minimal stack usage
        aggregator.admin = params.admin;
        aggregator.total_protocols = 0;
        aggregator.total_tvl = 0;
        aggregator.total_yield_earned = 0;
        aggregator.emergency_paused = false;
        aggregator.fee_rate = 0;
        aggregator.fee_recipient = params.admin;
        aggregator.bump = ctx.bumps.yield_aggregator;

        // Emit event with minimal stack usage
        emit!(YieldAggregatorInitialized {
            admin: params.admin,
            fee_rate: 0,
            fee_recipient: params.admin,
            timestamp,
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
        // Validate parameters early to minimize stack usage
        require!(params.risk_score >= 1 && params.risk_score <= 10, YieldAggregatorError::InvalidRiskScore);
        require!(params.name.len() <= 32, YieldAggregatorError::InvalidProtocolName);

        let timestamp = Clock::get()?.unix_timestamp;
        
        // Initialize protocol info with minimal stack usage
        let protocol = &mut ctx.accounts.protocol_info;
        protocol.name = params.name.clone();
        protocol.chain_id = params.chain_id;
        protocol.current_apy = params.initial_apy;
        protocol.tvl = 0;
        protocol.max_capacity = params.max_capacity;
        protocol.risk_score = params.risk_score;
        protocol.is_active = true;
        protocol.last_update = timestamp;
        protocol.bump = ctx.bumps.protocol_info;

        // Update aggregator
        ctx.accounts.yield_aggregator.total_protocols += 1;

        // Emit event
        emit!(ProtocolAdded {
            name: params.name.clone(),
            chain_id: params.chain_id,
            apy: params.initial_apy,
            max_capacity: params.max_capacity,
            risk_score: params.risk_score,
            timestamp,
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

        let timestamp = Clock::get()?.unix_timestamp;
        let user_key = ctx.accounts.user.key();
        
        // Update user position with minimal stack usage
        let user_position = &mut ctx.accounts.user_position;
        user_position.user = user_key;
        user_position.total_deposits += params.amount;
        user_position.position_count += 1;
        user_position.last_activity = timestamp;
        user_position.bump = ctx.bumps.user_position;

        // Update total TVL
        ctx.accounts.yield_aggregator.total_tvl += params.amount;

        // Emit event
        emit!(CrossChainDepositRequested {
            user: user_key,
            amount: params.amount,
            target_chain: params.target_chain_id,
            target_protocol: params.target_protocol.clone(),
            timestamp,
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

        let timestamp = Clock::get()?.unix_timestamp;
        let user_key = ctx.accounts.user.key();
        
        // Update user position
        ctx.accounts.user_position.last_activity = timestamp;

        // Emit event
        emit!(CrossChainWithdrawRequested {
            user: user_key,
            amount: params.amount,
            target_chain: params.target_chain_id,
            timestamp,
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

        let timestamp = Clock::get()?.unix_timestamp;
        let user_key = ctx.accounts.user.key();
        
        // Update user position
        ctx.accounts.user_position.last_activity = timestamp;

        // Emit event
        emit!(RebalanceRequested {
            user: user_key,
            from_protocol: params.from_protocol.clone(),
            to_protocol: params.to_protocol.clone(),
            amount: params.amount,
            target_chain: params.target_chain_id,
            timestamp,
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
        let timestamp = Clock::get()?.unix_timestamp;
        
        // Update protocol info
        let protocol = &mut ctx.accounts.protocol_info;
        protocol.current_apy = params.new_apy;
        protocol.last_update = timestamp;

        // Emit event
        emit!(YieldRateUpdated {
            protocol: params.protocol_name.clone(),
            new_apy: params.new_apy,
            timestamp,
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

        let timestamp = Clock::get()?.unix_timestamp;
        let user_key = ctx.accounts.user.key();
        let yield_amount = user_position.total_yield_earned;
        
        // Update user position
        user_position.total_deposits += yield_amount;
        user_position.total_yield_earned = 0;
        user_position.last_activity = timestamp;

        // Emit event
        emit!(YieldCompounded {
            user: user_key,
            protocol: params.protocol_name.clone(),
            yield_amount,
            new_principal: user_position.total_deposits,
            timestamp,
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
        let timestamp = Clock::get()?.unix_timestamp;
        let admin_key = ctx.accounts.admin.key();
        
        // Update emergency pause state
        ctx.accounts.yield_aggregator.emergency_paused = params.pause;

        // Emit appropriate event
        if params.pause {
            emit!(EmergencyPauseActivated {
                admin: admin_key,
                timestamp,
            });
        } else {
            emit!(EmergencyPauseDeactivated {
                admin: admin_key,
                timestamp,
            });
        }

        Ok(())
    }
}

// Note: GetOptimalStrategy is removed to reduce stack usage.
// This functionality should be implemented client-side by fetching protocol data 
// and calculating the optimal strategy based on user preferences.