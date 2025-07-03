use crate::*;
use anchor_lang::prelude::*;
use oapp::endpoint::{
    instructions::ClearParams, state::EndpointSettings, ENDPOINT_SEED, ID as ENDPOINT_ID,
};
use oapp::LzReceiveParams;

#[derive(Accounts)]
#[instruction(params: LzReceiveParams)]
pub struct LzReceive<'info> {
    #[account(seeds = [STORE_SEED], bump = store.bump)]
    pub store: Account<'info, OAppStore>,
    #[account(
        seeds = [ENDPOINT_SEED],
        bump = endpoint.bump,
        seeds::program = ENDPOINT_ID
    )]
    pub endpoint: Account<'info, EndpointSettings>,
    #[account(
        seeds = [
            PEER_SEED,
            &store.key().to_bytes(),
            &params.src_eid.to_be_bytes()
        ],
        bump = peer.bump
    )]
    pub peer: Account<'info, OAppPeerConfig>
}

impl<'info> LzReceive<'info> {
    pub fn apply(ctx: &mut Context<LzReceive>, params: &LzReceiveParams) -> Result<()> {
        let message = msg_codec::decode(&params.message)?;
        msg!("Received message: {}", message);
        
        // Clear the message from the endpoint
        let clear_params = ClearParams {
            receiver: ctx.accounts.store.key(),
            src_eid: params.src_eid,
            sender: params.sender,
            nonce: params.nonce,
        };
        
        oapp::endpoint_cpi::clear(
            ENDPOINT_ID,
            ctx.accounts.store.key(),
            ctx.remaining_accounts,
            clear_params,
        )?;
        
        Ok(())
    }
}