use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::*};
use crate::{YIELD_AGGREGATOR_SEED, USER_POSITION_SEED};

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