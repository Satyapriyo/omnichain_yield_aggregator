use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::YieldAggregatorError};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EmergencyPauseParams {
    pub pause: bool,
}

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(
        mut,
        seeds = [crate::YIELD_AGGREGATOR_SEED],
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