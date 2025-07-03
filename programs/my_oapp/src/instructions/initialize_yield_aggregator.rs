use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::*};
use crate::YIELD_AGGREGATOR_SEED;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeYieldAggregatorParams {
    pub admin: Pubkey,
}

#[derive(Accounts)]
pub struct InitializeYieldAggregator<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + YieldAggregator::INIT_SPACE,
        seeds = [YIELD_AGGREGATOR_SEED],
        bump
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl InitializeYieldAggregator<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &InitializeYieldAggregatorParams) -> Result<()> {
        let yield_aggregator = &mut ctx.accounts.yield_aggregator;
        yield_aggregator.admin = params.admin;
        yield_aggregator.total_protocols = 0;
        yield_aggregator.total_tvl = 0;
        yield_aggregator.total_yield_earned = 0;
        yield_aggregator.emergency_paused = false;
        yield_aggregator.bump = ctx.bumps.yield_aggregator;

        emit!(YieldAggregatorInitialized {
            admin: params.admin,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}