use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::*};
use crate::{YIELD_AGGREGATOR_SEED, PROTOCOL_SEED};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AddProtocolParams {
    pub name: String,
    pub chain_id: u32,
    pub initial_apy: u64,
    pub max_capacity: u64,
    pub risk_score: u8,
}

#[derive(Accounts)]
#[instruction(params: AddProtocolParams)]
pub struct AddProtocol<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + ProtocolInfo::INIT_SPACE,
        seeds = [PROTOCOL_SEED, params.name.as_bytes()],
        bump
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

impl AddProtocol<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &AddProtocolParams) -> Result<()> {
        require!(params.risk_score >= 1 && params.risk_score <= 10, YieldAggregatorError::InvalidRiskScore);
        require!(params.name.len() <= 32, YieldAggregatorError::InvalidProtocolName);

        let protocol_info = &mut ctx.accounts.protocol_info;
        protocol_info.name = params.name.clone();
        protocol_info.chain_id = params.chain_id;
        protocol_info.current_apy = params.initial_apy;
        protocol_info.tvl = 0;
        protocol_info.max_capacity = params.max_capacity;
        protocol_info.risk_score = params.risk_score;
        protocol_info.is_active = true;
        protocol_info.last_update = Clock::get()?.unix_timestamp;
        protocol_info.bump = ctx.bumps.protocol_info;

        let yield_aggregator = &mut ctx.accounts.yield_aggregator;
        yield_aggregator.total_protocols += 1;

        emit!(ProtocolAdded {
            name: params.name.clone(),
            chain_id: params.chain_id,
            apy: params.initial_apy,
            max_capacity: params.max_capacity,
            risk_score: params.risk_score,
        });

        Ok(())
    }
}