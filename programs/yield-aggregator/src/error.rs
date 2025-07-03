use anchor_lang::prelude::*;

#[error_code]
pub enum YieldAggregatorError {
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Protocol is inactive")]
    ProtocolInactive,
    #[msg("APY too low")]
    ApyTooLow,
    #[msg("Emergency pause is active")]
    EmergencyPaused,
    #[msg("Invalid risk score")]
    InvalidRiskScore,
    #[msg("Invalid protocol name")]
    InvalidProtocolName,
    #[msg("No yield to compound")]
    NoYieldToCompound,
    #[msg("Protocol capacity exceeded")]
    ProtocolCapacityExceeded,
    #[msg("Invalid chain ID")]
    InvalidChainId,
}
