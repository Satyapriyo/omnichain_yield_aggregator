use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*};

#[derive(Accounts)]
#[instruction(params: InitializeYieldAggregatorParams)]
pub struct InitializeYieldAggregator<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [crate::YIELD_AGGREGATOR_SEED],
        bump,
        space = YieldAggregator::SIZE,
    )]
    pub yield_aggregator: Account<'info, YieldAggregator>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeYieldAggregator<'info> {
    pub fn apply(
        ctx: &mut Context<InitializeYieldAggregator>,
        params: &InitializeYieldAggregatorParams,
    ) -> Result<()> {
        let yield_aggregator = &mut ctx.accounts.yield_aggregator;
        yield_aggregator.admin = ctx.accounts.admin.key();
        yield_aggregator.bump = ctx.bumps.yield_aggregator;
        yield_aggregator.total_value_locked = 0;
        yield_aggregator.protocol_count = 0;
        yield_aggregator.is_paused = false;
        yield_aggregator.fee_rate = params.fee_rate;
        yield_aggregator.fee_recipient = params.fee_recipient;
        
        emit!(YieldAggregatorInitialized {
            admin: ctx.accounts.admin.key(),
            fee_rate: params.fee_rate,
            fee_recipient: params.fee_recipient,
        });
        
        Ok(())
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeYieldAggregatorParams {
    pub fee_rate: u64,
    pub fee_recipient: Pubkey,
}