use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::*};
use crate::{YIELD_AGGREGATOR_SEED, PROTOCOL_SEED, USER_POSITION_SEED};

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