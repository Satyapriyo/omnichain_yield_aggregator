use anchor_lang::prelude::*;

#[event]
pub struct YieldAggregatorInitialized {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ProtocolAdded {
    pub name: String,
    pub chain_id: u32,
    pub apy: u64,
    pub max_capacity: u64,
    pub risk_score: u8,
}

#[event]
pub struct CrossChainDepositRequested {
    pub user: Pubkey,
    pub amount: u64,
    pub target_chain: u32,
    pub target_protocol: String,
    pub timestamp: i64,
}

#[event]
pub struct LocalDepositProcessed {
    pub user: Pubkey,
    pub amount: u64,
    pub protocol: String,
    pub apy: u64,
    pub timestamp: i64,
}

#[event]
pub struct CrossChainWithdrawRequested {
    pub user: Pubkey,
    pub amount: u64,
    pub target_chain: u32,
    pub timestamp: i64,
}

#[event]
pub struct YieldWithdrawn {
    pub user: Pubkey,
    pub amount: u64,
    pub target_chain: u32,
    pub timestamp: i64,
}

#[event]
pub struct RebalanceRequested {
    pub user: Pubkey,
    pub from_protocol: String,
    pub to_protocol: String,
    pub amount: u64,
    pub target_chain: u32,
    pub timestamp: i64,
}

#[event]
pub struct YieldRateUpdated {
    pub protocol: String,
    pub new_apy: u64,
    pub timestamp: i64,
}

#[event]
pub struct YieldCompounded {
    pub user: Pubkey,
    pub protocol: String,
    pub yield_amount: u64,
    pub new_principal: u64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyPauseActivated {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyPauseDeactivated {
    pub admin: Pubkey,
    pub timestamp: i64,
}