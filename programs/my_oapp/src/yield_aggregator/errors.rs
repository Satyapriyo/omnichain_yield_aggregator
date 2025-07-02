use anchor_lang::prelude::*;

#[error_code]
pub enum YieldAggregatorError {
    #[msg("Invalid amount provided")]
    InvalidAmount,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("APY too low")]
    ApyTooLow,
    #[msg("Protocol is inactive")]
    ProtocolInactive,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Emergency pause is active")]
    EmergencyPaused,
    #[msg("Invalid risk score (must be 1-10)")]
    InvalidRiskScore,
    #[msg("Protocol capacity exceeded")]
    CapacityExceeded,
    #[msg("Invalid protocol name")]
    InvalidProtocolName,
    #[msg("Position not found")]
    PositionNotFound,
    #[msg("No yield to compound")]
    NoYieldToCompound,
}