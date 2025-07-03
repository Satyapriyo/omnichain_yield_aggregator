use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::YieldAggregatorError};
use crate::errors::MyOAppError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RebalancePositionParams {
    pub from_protocol: String,
    pub to_protocol: String,
    pub amount: u64,
    pub min_apy_improvement: u64,
}

#[derive(Accounts)]
#[instruction(params: RebalancePositionParams)]
pub struct RebalancePosition<'info> {
    #[account(
        mut,
        seeds = [crate::USER_POSITION_SEED, user.key().as_ref()],
        bump = user_position.bump,
        has_one = user @ YieldAggregatorError::Unauthorized
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(
        seeds = [crate::PROTOCOL_SEED, params.from_protocol.as_bytes()],
        bump = from_protocol.bump,
        constraint = from_protocol.is_active @ YieldAggregatorError::ProtocolInactive
    )]
    pub from_protocol: Account<'info, ProtocolInfo>,
    #[account(
        seeds = [crate::PROTOCOL_SEED, params.to_protocol.as_bytes()],
        bump = to_protocol.bump,
        constraint = to_protocol.is_active @ YieldAggregatorError::ProtocolInactive,
        constraint = to_protocol.current_apy > from_protocol.current_apy + params.min_apy_improvement @ MyOAppError::RebalanceNotNeeded
    )]
    pub to_protocol: Account<'info, ProtocolInfo>,
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

impl RebalancePosition<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &RebalancePositionParams) -> Result<()> {
        require!(params.amount > 0, YieldAggregatorError::InvalidAmount);
        require!(
            ctx.accounts.user_position.total_deposits >= params.amount,
            MyOAppError::WithdrawalExceedsBalance
        );

        let timestamp = Clock::get()?.unix_timestamp;
        let user_key = ctx.accounts.user.key();
        
        // Update user position activity
        let user_position = &mut ctx.accounts.user_position;
        user_position.last_activity = timestamp;

        // Emit event for cross-chain rebalancing
        emit!(RebalanceRequested {
            user: user_key,
            from_protocol: params.from_protocol.clone(),
            to_protocol: params.to_protocol.clone(),
            amount: params.amount,
            target_chain: ctx.accounts.from_protocol.chain_id,
            timestamp,
        });

        Ok(())
    }
}