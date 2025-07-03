use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::*};
use crate::{YIELD_AGGREGATOR_SEED, PROTOCOL_SEED};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateYieldRatesParams {
    pub protocol_name: String,
    pub new_apy: u64,
}

#[derive(Accounts)]
#[instruction(params: UpdateYieldRatesParams)]
pub struct UpdateYieldRates<'info> {
    #[account(
        mut,
        seeds = [PROTOCOL_SEED, params.protocol_name.as_bytes()],
        bump = protocol_info.bump
    )]
    pub protocol_info: Account<'info, ProtocolInfo>,
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

impl UpdateYieldRates<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &UpdateYieldRatesParams) -> Result<()> {
        let protocol_info = &mut ctx.accounts.protocol_info;
        protocol_info.current_apy = params.new_apy;
        protocol_info.last_update = Clock::get()?.unix_timestamp;

        emit!(YieldRateUpdated {
            protocol: params.protocol_name.clone(),
            new_apy: params.new_apy,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}