use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct YieldAggregator {
    pub admin: Pubkey,
    pub total_protocols: u32,
    pub total_tvl: u64,
    pub total_yield_earned: u64,
    pub emergency_paused: bool,
    pub bump: u8,
    pub fee_rate: u64,
    pub fee_recipient: Pubkey,
}

impl YieldAggregator {
    pub const SIZE: usize = 8 + 32 + 4 + 8 + 8 + 1 + 1 + 8 + 32; // discriminator + fields
}

#[account]
#[derive(InitSpace)]
pub struct ProtocolInfo {
    #[max_len(32)]
    pub name: String,
    pub chain_id: u32,
    pub current_apy: u64, // Basis points (10000 = 100%)
    pub tvl: u64,
    pub max_capacity: u64,
    pub risk_score: u8, // 1-10 scale
    pub is_active: bool,
    pub last_update: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserPosition {
    pub user: Pubkey,
    pub total_deposits: u64,
    pub total_yield_earned: u64,
    pub position_count: u32,
    pub last_activity: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct YieldVault {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub total_deposits: u64,
    pub bump: u8,
}