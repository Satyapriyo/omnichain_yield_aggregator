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

    pub fn add_protocol(
        mut ctx: Context<AddProtocol>,
        params: AddProtocolParams,
    ) -> Result<()> {
        AddProtocol::apply(&mut ctx, &params)
    }

    pub fn deposit_for_yield(
        mut ctx: Context<DepositForYield>,
        params: DepositForYieldParams,
    ) -> Result<()> {
        DepositForYield::apply(&mut ctx, &params)
    }

    pub fn emergency_pause(
        mut ctx: Context<EmergencyPause>,
        params: EmergencyPauseParams,
    ) -> Result<()> {
        EmergencyPause::apply(&mut ctx, &params)
    }

    pub fn update_yield_rates(
        mut ctx: Context<UpdateYieldRates>,
        params: UpdateYieldRatesParams,
    ) -> Result<()> {
        UpdateYieldRates::apply(&mut ctx, &params)
    }
}