use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::*};
use crate::YIELD_AGGREGATOR_SEED;

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