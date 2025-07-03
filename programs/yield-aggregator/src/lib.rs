use anchor_lang::prelude::*;

declare_id!("59EhuyPwcVDy7YQtgM8WiXsjiG62QRhSRFqHkAHRCTnS");

pub const YIELD_AGGREGATOR_SEED: &[u8] = b"YieldAggregator";
pub const PROTOCOL_SEED: &[u8] = b"Protocol";
pub const USER_POSITION_SEED: &[u8] = b"UserPosition";
pub const YIELD_VAULT_SEED: &[u8] = b"YieldVault";

// ============================== State Structures ==============================

#[account]
#[derive(InitSpace)]
pub struct YieldAggregator {
    pub admin: Pubkey,
    pub total_protocols: u32,
    pub total_tvl: u64,
    pub total_yield_earned: u64,
    pub emergency_paused: bool,
    pub bump: u8,
    pub fee_rate: u64,
    pub fee_recipient: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct ProtocolInfo {
    #[max_len(32)]
    pub name: String,
    pub chain_id: u32,
    pub current_apy: u64,
    pub tvl: u64,
    pub max_capacity: u64,
    pub risk_score: u8,
    pub is_active: bool,
    pub last_update: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserPosition {
    pub user: Pubkey,
    pub total_deposits: u64,
    pub total_yield_earned: u64,
    pub position_count: u32,
    pub last_activity: i64,
    pub bump: u8,
}

// ============================== Error Codes ==============================

#[error_code]
pub enum YieldAggregatorError {
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Protocol is inactive")]
    ProtocolInactive,
    #[msg("APY too low")]
    ApyTooLow,
    #[msg("Emergency pause is active")]
    EmergencyPaused,
    #[msg("Invalid risk score")]
    InvalidRiskScore,
    #[msg("Invalid protocol name")]
    InvalidProtocolName,
    #[msg("No yield to compound")]
    NoYieldToCompound,
}

// ============================== Instruction Parameters ==============================

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeYieldAggregatorParams {
    pub admin: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AddProtocolParams {
    pub name: String,
    pub chain_id: u32,
    pub initial_apy: u64,
    pub max_capacity: u64,
    pub risk_score: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DepositForYieldParams {
    pub amount: u64,
    pub target_protocol: String,
    pub min_apy: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawYieldParams {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateYieldRatesParams {
    pub protocol_name: String,
    pub new_apy: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EmergencyPauseParams {
    pub pause: bool,
}

// ============================== Instructions ==============================

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

#[program]
pub mod yield_aggregator {
    use super::*;

    pub fn initialize_yield_aggregator(
        ctx: Context<InitializeYieldAggregator>, 
        params: InitializeYieldAggregatorParams
    ) -> Result<()> {
        let aggregator = &mut ctx.accounts.yield_aggregator;
        
        aggregator.admin = params.admin;
        aggregator.total_protocols = 0;
        aggregator.total_tvl = 0;
        aggregator.total_yield_earned = 0;
        aggregator.emergency_paused = false;
        aggregator.fee_rate = 0;
        aggregator.fee_recipient = params.admin;
        aggregator.bump = ctx.bumps.yield_aggregator;

        Ok(())
    }

    pub fn add_protocol(
        ctx: Context<AddProtocol>,
        params: AddProtocolParams,
    ) -> Result<()> {
        require!(params.risk_score >= 1 && params.risk_score <= 10, YieldAggregatorError::InvalidRiskScore);
        require!(params.name.len() <= 32, YieldAggregatorError::InvalidProtocolName);

        let timestamp = Clock::get()?.unix_timestamp;
        
        let protocol = &mut ctx.accounts.protocol_info;
        protocol.name = params.name;
        protocol.chain_id = params.chain_id;
        protocol.current_apy = params.initial_apy;
        protocol.tvl = 0;
        protocol.max_capacity = params.max_capacity;
        protocol.risk_score = params.risk_score;
        protocol.is_active = true;
        protocol.last_update = timestamp;
        protocol.bump = ctx.bumps.protocol_info;

        ctx.accounts.yield_aggregator.total_protocols += 1;

        Ok(())
    }

    pub fn deposit_for_yield(
        ctx: Context<DepositForYield>,
        params: DepositForYieldParams,
    ) -> Result<()> {
        require!(params.amount > 0, YieldAggregatorError::InvalidAmount);

        let timestamp = Clock::get()?.unix_timestamp;
        let user_key = ctx.accounts.user.key();
        
        let user_position = &mut ctx.accounts.user_position;
        user_position.user = user_key;
        user_position.total_deposits += params.amount;
        user_position.position_count += 1;
        user_position.last_activity = timestamp;
        user_position.bump = ctx.bumps.user_position;

        ctx.accounts.yield_aggregator.total_tvl += params.amount;

        Ok(())
    }

    pub fn withdraw_yield(
        ctx: Context<WithdrawYield>,
        params: WithdrawYieldParams,
    ) -> Result<()> {
        require!(params.amount > 0, YieldAggregatorError::InvalidAmount);

        let timestamp = Clock::get()?.unix_timestamp;
        ctx.accounts.user_position.last_activity = timestamp;

        Ok(())
    }

    pub fn update_yield_rates(
        ctx: Context<UpdateYieldRates>,
        params: UpdateYieldRatesParams,
    ) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;
        
        let protocol = &mut ctx.accounts.protocol_info;
        protocol.current_apy = params.new_apy;
        protocol.last_update = timestamp;

        Ok(())
    }

    pub fn emergency_pause(
        ctx: Context<EmergencyPause>,
        params: EmergencyPauseParams,
    ) -> Result<()> {
        ctx.accounts.yield_aggregator.emergency_paused = params.pause;
        Ok(())
    }
}
