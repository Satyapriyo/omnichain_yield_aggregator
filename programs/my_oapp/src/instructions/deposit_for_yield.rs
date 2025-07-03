use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::YieldAggregatorError};
use crate::errors::MyOAppError;

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
        seeds = [crate::USER_POSITION_SEED, user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(
        seeds = [crate::PROTOCOL_SEED, params.target_protocol.as_bytes()],
        bump = protocol_info.bump,
        constraint = protocol_info.is_active @ YieldAggregatorError::ProtocolInactive,
        constraint = protocol_info.current_apy >= params.min_apy @ YieldAggregatorError::ApyTooLow
    )]
    pub protocol_info: Account<'info, ProtocolInfo>,
    #[account(
        mut,
        seeds = [crate::YIELD_AGGREGATOR_SEED],
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
        require!(params.amount >= 1000, MyOAppError::MinimumDepositNotMet); // Minimum deposit check

        let timestamp = Clock::get()?.unix_timestamp;
        let user_key = ctx.accounts.user.key();
        
        // Update user position
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