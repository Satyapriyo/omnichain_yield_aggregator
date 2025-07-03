use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::YieldAggregatorError};
use crate::errors::MyOAppError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawYieldParams {
    pub amount: u64,
    pub protocol_name: String,
    pub target_chain_id: u32,
}

#[derive(Accounts)]
#[instruction(params: WithdrawYieldParams)]
pub struct WithdrawYield<'info> {
    #[account(
        mut,
        seeds = [crate::USER_POSITION_SEED, user.key().as_ref()],
        bump = user_position.bump,
        has_one = user @ YieldAggregatorError::Unauthorized
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(
        seeds = [crate::PROTOCOL_SEED, params.protocol_name.as_bytes()],
        bump = protocol_info.bump,
        constraint = protocol_info.is_active @ YieldAggregatorError::ProtocolInactive
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

impl WithdrawYield<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &WithdrawYieldParams) -> Result<()> {
        require!(params.amount > 0, YieldAggregatorError::InvalidAmount);
        require!(
            ctx.accounts.user_position.total_deposits >= params.amount,
            MyOAppError::WithdrawalExceedsBalance
        );

        let timestamp = Clock::get()?.unix_timestamp;
        let user_key = ctx.accounts.user.key();
        
        // Update user position
        let user_position = &mut ctx.accounts.user_position;
        user_position.total_deposits -= params.amount;
        user_position.last_activity = timestamp;

        // Update total TVL
        ctx.accounts.yield_aggregator.total_tvl -= params.amount;

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