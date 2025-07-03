pub mod instructions;
pub mod state;
pub mod events;
pub mod errors;

pub use instructions::*;
pub use state::*;
pub use events::*;
pub use errors::*;

use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeYieldAggregatorParams {
    pub authority: Pubkey,
    pub fee_rate: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AddProtocolParams {
    pub name: String,
    pub protocol_address: Pubkey,
    pub initial_yield_rate: u64,
    pub max_capacity: u64,
    pub risk_level: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DepositForYieldParams {
    pub protocol_id: u64,
    pub amount: u64,
    pub min_yield_rate: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawYieldParams {
    pub protocol_id: u64,
    pub amount: u64,
    pub withdraw_yield: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RebalancePositionParams {
    pub from_protocol_id: u64,
    pub to_protocol_id: u64,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateYieldRatesParams {
    pub protocol_id: u64,
    pub new_yield_rate: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CompoundYieldParams {
    pub protocol_id: u64,
    pub reinvest_yield: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EmergencyPauseParams {
    pub pause: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct GetOptimalStrategyParams {
    pub amount: u64,
    pub risk_tolerance: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct OptimalStrategyResponse {
    pub recommended_protocol_id: u64,
    pub expected_yield: u64,
    pub risk_score: u8,
}