use anchor_lang::prelude::*;
use crate::yield_aggregator::{state::*, events::*, errors::YieldAggregatorError};

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
        seeds = [crate::PROTOCOL_SEED, params.name.as_bytes()],
        bump
    )]
    pub protocol_info: Account<'info, ProtocolInfo>,
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

impl AddProtocol<'_> {
    pub fn apply(ctx: &mut Context<Self>, params: &AddProtocolParams) -> Result<()> {
        // Validate parameters early to minimize stack usage
        require!(params.risk_score >= 1 && params.risk_score <= 10, YieldAggregatorError::InvalidRiskScore);
        require!(params.name.len() <= 32, YieldAggregatorError::InvalidProtocolName);

        let timestamp = Clock::get()?.unix_timestamp;
        
        // Initialize protocol info with minimal stack usage
        let protocol = &mut ctx.accounts.protocol_info;
        protocol.name = params.name.clone();
        protocol.chain_id = params.chain_id;
        protocol.current_apy = params.initial_apy;
        protocol.tvl = 0;
        protocol.max_capacity = params.max_capacity;
        protocol.risk_score = params.risk_score;
        protocol.is_active = true;
        protocol.last_update = timestamp;
        protocol.bump = ctx.bumps.protocol_info;

        // Update aggregator
        ctx.accounts.yield_aggregator.total_protocols += 1;

        // Emit event
        emit!(ProtocolAdded {
            name: params.name.clone(),
            chain_id: params.chain_id,
            apy: params.initial_apy,
            max_capacity: params.max_capacity,
            risk_score: params.risk_score,
            timestamp,
        });

        Ok(())
    }
}