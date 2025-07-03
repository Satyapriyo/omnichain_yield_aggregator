use anchor_lang::prelude::*;

#[account]
pub struct YieldAggregator {
    pub authority: Pubkey,
    pub total_protocols: u64,
    pub total_value_locked: u64,
    pub fee_rate: u64, // basis points
    pub is_paused: bool,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
pub struct Protocol {
    pub id: u64,
    pub name: String,
    pub protocol_address: Pubkey,
    pub current_yield_rate: u64, // basis points
    pub total_deposited: u64,
    pub max_capacity: u64,
    pub is_active: bool,
    pub risk_level: u8, // 1-5 scale
    pub last_yield_update: i64,
    pub bump: u8,
}

#[account]
pub struct UserPosition {
    pub user: Pubkey,
    pub protocol_id: u64,
    pub deposited_amount: u64,
    pub earned_yield: u64,
    pub last_compound_time: i64,
    pub entry_yield_rate: u64,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
pub struct YieldVault {
    pub token_mint: Pubkey,
    pub vault_authority: Pubkey,
    pub total_shares: u64,
    pub total_assets: u64,
    pub last_harvest_time: i64,
    pub performance_fee: u64, // basis points
    pub bump: u8,
}