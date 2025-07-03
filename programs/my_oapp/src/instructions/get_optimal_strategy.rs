use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*};

// Note: GetOptimalStrategy is implemented as a view function
// that returns strategy recommendations without state changes
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct GetOptimalStrategyParams {
    pub amount: u64,
    pub risk_tolerance: u8, // 1-10 scale
    pub min_apy: u64,
}

#[derive(Accounts)]
#[instruction(params: GetOptimalStrategyParams)]
pub struct GetOptimalStrategy<'info> {
    #[account(
        seeds = [crate::YIELD_AGGREGATOR_SEED],
        bump = yield_aggregator.bump
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    pub user: Signer<'info>,
}

impl GetOptimalStrategy<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &GetOptimalStrategyParams) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;
        let user_key = ctx.accounts.user.key();

        // Emit event with strategy request (actual calculation would be done off-chain)
        emit!(OptimalStrategyRequested {
            user: user_key,
            amount: params.amount,
            risk_tolerance: params.risk_tolerance,
            min_apy: params.min_apy,
            timestamp,
        });

        // This is a view-like function that doesn't modify state
        // The actual strategy calculation should be done client-side
        // by fetching protocol data and calculating optimal allocation
        
        Ok(())
    }
}