use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::*};
use crate::{YIELD_AGGREGATOR_SEED, PROTOCOL_SEED, USER_POSITION_SEED};

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