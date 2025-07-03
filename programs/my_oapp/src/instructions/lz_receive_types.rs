use crate::*;
use anchor_lang::prelude::*;
use oapp::endpoint::{
    state::EndpointSettings, ENDPOINT_SEED, ID as ENDPOINT_ID,
};
use oapp::LzReceiveParams;

#[derive(Accounts)]
#[instruction(params: LzReceiveParams)]
pub struct LzReceiveTypes<'info> {
    #[account(seeds = [STORE_SEED], bump = store.bump)]
    pub store: Account<'info, OAppStore>,
    #[account(
        seeds = [ENDPOINT_SEED],
        bump = endpoint.bump,
        seeds::program = ENDPOINT_ID
    )]
    pub endpoint: Account<'info, EndpointSettings>,
}

impl<'info> LzReceiveTypes<'info> {
    pub fn apply(
        ctx: &Context<LzReceiveTypes>,
        params: &LzReceiveParams,
    ) -> Result<Vec<oapp::endpoint_cpi::LzAccount>> {
        let accounts = vec![
            oapp::endpoint_cpi::LzAccount {
                pubkey: ctx.accounts.store.key(),
                is_signer: false,
                is_writable: false,
            },
        ];
        Ok(accounts)
    }
}