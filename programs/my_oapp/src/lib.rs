mod errors;
mod instructions;
mod msg_codec;
mod state;
mod yield_aggregator;

use anchor_lang::prelude::*;
use instructions::*;
use oapp::{endpoint::MessagingFee, LzReceiveParams};
use solana_helper::program_id_from_env;
use state::*;

declare_id!(anchor_lang::solana_program::pubkey::Pubkey::new_from_array(program_id_from_env!(
    "MYOAPP_ID",
    "41NCdrEvXhQ4mZgyJkmqYxL6A1uEmnraGj31UJ6PsXd3"
)));

const LZ_RECEIVE_TYPES_SEED: &[u8] = b"LzReceiveTypes";
const STORE_SEED: &[u8] = b"Store";
const PEER_SEED: &[u8] = b"Peer";

// Yield aggregator seeds
pub const YIELD_AGGREGATOR_SEED: &[u8] = b"YieldAggregator";
pub const PROTOCOL_SEED: &[u8] = b"Protocol";
pub const USER_POSITION_SEED: &[u8] = b"UserPosition";
pub const YIELD_VAULT_SEED: &[u8] = b"YieldVault";

#[program]
pub mod my_oapp {
    use super::*;

    // ============================== Original LayerZero Instructions ==============================
    pub fn init_store(mut ctx: Context<InitStore>, params: InitStoreParams) -> Result<()> {
        InitStore::apply(&mut ctx, &params)
    }

    pub fn set_peer_config(
        mut ctx: Context<SetPeerConfig>,
        params: SetPeerConfigParams,
    ) -> Result<()> {
        SetPeerConfig::apply(&mut ctx, &params)
    }

    pub fn quote_send(ctx: Context<QuoteSend>, params: QuoteSendParams) -> Result<MessagingFee> {
        QuoteSend::apply(&ctx, &params)
    }

    pub fn send(mut ctx: Context<Send>, params: SendMessageParams) -> Result<()> {
        Send::apply(&mut ctx, &params)
    }

    pub fn lz_receive(mut ctx: Context<LzReceive>, params: LzReceiveParams) -> Result<()> {
        LzReceive::apply(&mut ctx, &params)
    }

    pub fn lz_receive_types(
        ctx: Context<LzReceiveTypes>,
        params: LzReceiveParams,
    ) -> Result<Vec<oapp::endpoint_cpi::LzAccount>> {
        LzReceiveTypes::apply(&ctx, &params)
    }

    // ============================== Yield Aggregator Instructions ==============================
    
    pub fn initialize_yield_aggregator(
        mut ctx: Context<InitializeYieldAggregator>, 
        params: InitializeYieldAggregatorParams
    ) -> Result<()> {
        InitializeYieldAggregator::apply(&mut ctx, &params)
    }

    pub fn yield_add_protocol(
        mut ctx: Context<YieldAddProtocol>,
        params: YieldAddProtocolParams,
    ) -> Result<()> {
        YieldAddProtocol::apply(&mut ctx, &params)
    }

    pub fn yield_deposit_for_yield(
        mut ctx: Context<YieldDepositForYield>,
        params: YieldDepositForYieldParams,
    ) -> Result<()> {
        YieldDepositForYield::apply(&mut ctx, &params)
    }

    pub fn yield_withdraw_yield(
        mut ctx: Context<YieldWithdrawYield>,
        params: YieldWithdrawYieldParams,
    ) -> Result<()> {
        YieldWithdrawYield::apply(&mut ctx, &params)
    }

    pub fn yield_rebalance_position(
        mut ctx: Context<YieldRebalancePosition>,
        params: YieldRebalancePositionParams,
    ) -> Result<()> {
        YieldRebalancePosition::apply(&mut ctx, &params)
    }

    pub fn yield_update_yield_rates(
        mut ctx: Context<YieldUpdateYieldRates>,
        params: YieldUpdateYieldRatesParams,
    ) -> Result<()> {
        YieldUpdateYieldRates::apply(&mut ctx, &params)
    }

    pub fn yield_compound_yield(
        mut ctx: Context<YieldCompoundYield>,
        params: YieldCompoundYieldParams,
    ) -> Result<()> {
        YieldCompoundYield::apply(&mut ctx, &params)
    }

    pub fn yield_emergency_pause(
        mut ctx: Context<YieldEmergencyPause>,
        params: YieldEmergencyPauseParams,
    ) -> Result<()> {
        YieldEmergencyPause::apply(&mut ctx, &params)
    }

    pub fn yield_get_optimal_strategy(
        mut ctx: Context<YieldGetOptimalStrategy>,
        params: YieldGetOptimalStrategyParams,
    ) -> Result<()> {
        YieldGetOptimalStrategy::apply(&mut ctx, &params)
    }
}