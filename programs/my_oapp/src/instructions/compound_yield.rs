use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::YieldAggregatorError};
use crate::errors::MyOAppError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CompoundYieldParams {
    pub protocol_name: String,
}

#[derive(Accounts)]
#[instruction(params: CompoundYieldParams)]
pub struct CompoundYield<'info> {
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

impl CompoundYield<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &CompoundYieldParams) -> Result<()> {
        require!(
            ctx.accounts.user_position.total_yield_earned > 0,
            YieldAggregatorError::NoYieldToCompound
        );

        let timestamp = Clock::get()?.unix_timestamp;
        let user_key = ctx.accounts.user.key();
        
        // Simulate compounding by moving yield to deposits
        let user_position = &mut ctx.accounts.user_position;
        let yield_to_compound = user_position.total_yield_earned;
        user_position.total_deposits += yield_to_compound;
        user_position.total_yield_earned = 0; // Reset yield after compounding
        user_position.last_activity = timestamp;

        // Update aggregator TVL
        ctx.accounts.yield_aggregator.total_tvl += yield_to_compound;

        // Emit event
        emit!(YieldCompounded {
            user: user_key,
            protocol: params.protocol_name.clone(),
            yield_amount: yield_to_compound,
            new_principal: user_position.total_deposits,
            timestamp,
        });

        Ok(())
    }
}