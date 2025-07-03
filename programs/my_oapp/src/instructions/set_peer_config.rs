use crate::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(params: SetPeerConfigParams)]
pub struct SetPeerConfig<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [PEER_SEED, &store.key().to_bytes(), &params.dst_eid.to_be_bytes()],
        bump,
        space = OAppPeerConfig::SIZE,
    )]
    pub peer: Account<'info, OAppPeerConfig>,
    #[account(seeds = [STORE_SEED], bump = store.bump)]
    pub store: Account<'info, OAppStore>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> SetPeerConfig<'info> {
    pub fn apply(ctx: &mut Context<SetPeerConfig>, params: &SetPeerConfigParams) -> Result<()> {
        let peer = &mut ctx.accounts.peer;
        peer.peer_address = params.peer_address;
        peer.bump = ctx.bumps.peer;
        peer.enforced_options = params.enforced_options.clone();
        Ok(())
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SetPeerConfigParams {
    pub dst_eid: u32,
    pub peer_address: [u8; 32],
    pub enforced_options: EnforcedOptions,
}