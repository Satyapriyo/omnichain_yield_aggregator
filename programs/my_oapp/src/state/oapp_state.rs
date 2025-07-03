use anchor_lang::prelude::*;

#[account]
pub struct Store {
    pub version: u8,
    pub admin: Pubkey,
    pub bump: u8,
}

#[account]
pub struct PeerConfig {
    pub peer_address: [u8; 32],
    pub bump: u8,
}

impl Store {
    pub const LEN: usize = 8 + 1 + 32 + 1; // discriminator + version + admin + bump
}

impl PeerConfig {
    pub const LEN: usize = 8 + 32 + 1; // discriminator + peer_address + bump
}