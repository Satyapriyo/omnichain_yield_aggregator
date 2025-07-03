use anchor_lang::prelude::*;

#[error_code]
pub enum YieldAggregatorError {
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid risk score")]
    InvalidRiskScore,
    #[msg("Invalid protocol name")]
    InvalidProtocolName,
    #[msg("Protocol is inactive")]
    ProtocolInactive,
    #[msg("APY is too low")]
    ApyTooLow,
    #[msg("Emergency paused")]
    EmergencyPaused,
    #[msg("No yield to compound")]
    NoYieldToCompound,
}