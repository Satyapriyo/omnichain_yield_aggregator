use anchor_lang::prelude::*;

#[account]
pub struct Store {
    pub version: u8,
    pub admin: Pubkey,
    pub bump: u8,
    pub endpoint_program: Pubkey,
    pub string: String,
}

#[account]
pub struct PeerConfig {
    pub peer_address: [u8; 32],
    pub bump: u8,
    pub enforced_options: EnforcedOptions,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EnforcedOptions {
    pub send: Vec<u8>,
    pub send_and_call: Vec<u8>,
}

impl EnforcedOptions {
    pub fn combine_options(
        &self,
        extra_options: &Option<Vec<u8>>,
        user_options: &[u8],
    ) -> Result<Vec<u8>> {
        let mut combined_options = self.send.clone();
        
        // Add extra options if provided
        if let Some(extra) = extra_options {
            combined_options.extend_from_slice(extra);
        }
        
        // Add user options
        combined_options.extend_from_slice(user_options);
        
        Ok(combined_options)
    }
}

impl Store {
    pub const SIZE: usize = 8 + 1 + 32 + 1 + 32 + 200; // discriminator + version + admin + bump + endpoint_program + string
}

impl PeerConfig {
    pub const SIZE: usize = 8 + 32 + 1 + 200; // discriminator + peer_address + bump + enforced_options
}