use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::YieldAggregatorError};
use crate::errors::MyOAppError;

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
        seeds = [crate::PROTOCOL_SEED, params.protocol_name.as_bytes()],
        bump = protocol_info.bump,
        constraint = protocol_info.is_active @ YieldAggregatorError::ProtocolInactive
    )]
    pub protocol_info: Account<'info, ProtocolInfo>,
    #[account(
        seeds = [crate::YIELD_AGGREGATOR_SEED],
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
        require!(params.new_apy > 0, MyOAppError::InvalidYieldRate);
        require!(params.new_apy <= 1000000, MyOAppError::InvalidYieldRate); // Max 10000% APY

        let timestamp = Clock::get()?.unix_timestamp;
        
        let protocol = &mut ctx.accounts.protocol_info;
        let old_apy = protocol.current_apy;
        protocol.current_apy = params.new_apy;
        protocol.last_update = timestamp;

        // Emit event
        emit!(YieldRateUpdated {
            protocol: params.protocol_name.clone(),
            new_apy: params.new_apy,
            timestamp,
        });

        Ok(())
    }
}