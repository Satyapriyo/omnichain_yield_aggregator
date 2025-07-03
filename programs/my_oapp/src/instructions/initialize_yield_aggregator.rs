use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::*;
use crate::errors::MyOAppError;
use crate::{YIELD_AGGREGATOR_SEED};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeYieldAggregatorParams {
    pub authority: Pubkey,
    pub fee_rate: u64, // basis points
}

#[derive(Accounts)]
#[instruction(params: InitializeYieldAggregatorParams)]
pub struct InitializeYieldAggregator<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<YieldAggregator>(),
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
        
        // Validate fee rate (max 10%)
        if params.fee_rate > 1000 {
            return Err(MyOAppError::InvalidYieldRate.into());
        }
        
        yield_aggregator.authority = params.authority;
        yield_aggregator.total_protocols = 0;
        yield_aggregator.total_value_locked = 0;
        yield_aggregator.fee_rate = params.fee_rate;
        yield_aggregator.is_paused = false;
        yield_aggregator.created_at = Clock::get()?.unix_timestamp;
        yield_aggregator.bump = ctx.bumps.yield_aggregator;
        
        msg!("Yield Aggregator initialized with authority: {}", params.authority);
        Ok(())
    }
}