use crate::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(params: InitStoreParams)]
pub struct InitStore<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [STORE_SEED],
        bump,
        space = OAppStore::SIZE,
    )]
    pub store: Account<'info, OAppStore>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitStore<'info> {
    pub fn apply(ctx: &mut Context<InitStore>, params: &InitStoreParams) -> Result<()> {
        let store = &mut ctx.accounts.store;
        store.version = 1;
        store.admin = ctx.accounts.admin.key();
        store.bump = ctx.bumps.store;
        store.endpoint_program = params.endpoint_program;
        store.string = params.string.clone();
        Ok(())
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitStoreParams {
    pub endpoint_program: Pubkey,
    pub string: String,
}