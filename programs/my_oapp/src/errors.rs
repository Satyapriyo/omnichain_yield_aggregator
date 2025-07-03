use anchor_lang::prelude::error_code;

#[error_code]
pub enum MyOAppError {
    #[msg("Invalid message type")]
    InvalidMessageType,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Invalid protocol")]
    InvalidProtocol,
    #[msg("Protocol not active")]
    ProtocolNotActive,
    #[msg("Position not found")]
    PositionNotFound,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Contract is paused")]
    ContractPaused,
    #[msg("Invalid yield rate")]
    InvalidYieldRate,
    #[msg("Rebalance not needed")]
    RebalanceNotNeeded,
    #[msg("Minimum deposit not met")]
    MinimumDepositNotMet,
    #[msg("Withdrawal exceeds balance")]
    WithdrawalExceedsBalance,
    #[msg("Protocol capacity exceeded")]
    ProtocolCapacityExceeded,
    #[msg("Invalid slippage tolerance")]
    InvalidSlippageTolerance,
    #[msg("Oracle price stale")]
    OraclePriceStale,
    #[msg("Cross-chain message failed")]
    CrossChainMessageFailed,
}