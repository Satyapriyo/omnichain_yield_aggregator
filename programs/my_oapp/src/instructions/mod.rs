pub mod send;
pub mod init_store;
pub mod lz_receive;
pub mod lz_receive_types;
pub mod quote_send;
pub mod set_peer_config;
pub mod initialize_yield_aggregator;
pub mod add_protocol;
pub mod deposit_for_yield;
pub mod withdraw_yield;
pub mod rebalance_position;
pub mod update_yield_rates;
pub mod compound_yield;
pub mod emergency_pause;
pub mod get_optimal_strategy;

pub use send::*;
pub use init_store::*;
pub use lz_receive::*;
pub use lz_receive_types::*;
pub use quote_send::*;
pub use set_peer_config::*;
pub use initialize_yield_aggregator::*;

// Re-export yield instruction structs with prefixes to avoid conflicts
pub use add_protocol::{AddProtocol as YieldAddProtocol, AddProtocolParams as YieldAddProtocolParams};
pub use deposit_for_yield::{DepositForYield as YieldDepositForYield, DepositForYieldParams as YieldDepositForYieldParams};
pub use withdraw_yield::{WithdrawYield as YieldWithdrawYield, WithdrawYieldParams as YieldWithdrawYieldParams};
pub use rebalance_position::{RebalancePosition as YieldRebalancePosition, RebalancePositionParams as YieldRebalancePositionParams};
pub use update_yield_rates::{UpdateYieldRates as YieldUpdateYieldRates, UpdateYieldRatesParams as YieldUpdateYieldRatesParams};
pub use compound_yield::{CompoundYield as YieldCompoundYield, CompoundYieldParams as YieldCompoundYieldParams};
pub use emergency_pause::{EmergencyPause as YieldEmergencyPause, EmergencyPauseParams as YieldEmergencyPauseParams};
pub use get_optimal_strategy::{GetOptimalStrategy as YieldGetOptimalStrategy, GetOptimalStrategyParams as YieldGetOptimalStrategyParams};