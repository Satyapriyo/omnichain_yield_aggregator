use anchor_lang::prelude::*;
use std::str;

// -----------------------------------------------------------------------------
// Enhanced Message Codec for Omnichain Yield Aggregator
// This file defines how the yield aggregator program encodes and decodes its 
// cross-chain messages. Each OApp can implement its own layout as long as the 
// sending and receiving chains agree.
// -----------------------------------------------------------------------------

// Message layout:
// Offset →
// 0        1        5                     37       41                     41+N
// |--------|--------|---------------------|--------|------------------------->
// | 1 byte | 4 bytes|     32 bytes       | 4 bytes|     N bytes            |
// |msg_type|version |    message_id      |  len   | serialized payload     |
// |--------|--------|---------------------|--------|------------------------|

pub const MSG_TYPE_OFFSET: usize = 0;
pub const VERSION_OFFSET: usize = 1;
pub const MESSAGE_ID_OFFSET: usize = 5;
pub const LENGTH_OFFSET: usize = 37;
pub const PAYLOAD_OFFSET: usize = 41;

// Legacy string message offsets
pub const STRING_LENGTH_OFFSET: usize = 0;
pub const STRING_PAYLOAD_OFFSET: usize = 32;

// Current protocol version
pub const PROTOCOL_VERSION: u32 = 1;

#[error_code]
pub enum MsgCodecError {
    #[msg("Buffer too short to contain the message header")]
    InvalidLength,
    #[msg("Header says payload is N bytes but buffer is smaller")]
    BodyTooShort,
    #[msg("Payload bytes aren't valid")]
    InvalidPayload,
    #[msg("Unsupported message type")]
    UnsupportedMessageType,
    #[msg("Unsupported protocol version")]
    UnsupportedVersion,
    #[msg("Invalid message ID")]
    InvalidMessageId,
    #[msg("Serialization failed")]
    SerializationError,
    #[msg("Deserialization failed")]
    DeserializationError,
    #[msg("Invalid UTF-8 string")]
    InvalidUtf8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum YieldMessage {
    /// Cross-chain deposit request
    DepositRequest {
        user: Pubkey,
        amount: u64,
        token_mint: Pubkey,
        target_protocol_id: u64,
        min_apy: u64,
        slippage_tolerance: u64, // basis points
        deadline: i64,
        referrer: Option<Pubkey>,
    },
    
    /// Cross-chain withdrawal request
    WithdrawRequest {
        user: Pubkey,
        amount: u64,
        token_mint: Pubkey,
        protocol_id: u64,
        target_chain_id: u32,
        destination_address: Vec<u8>,
        withdraw_yield: bool,
        deadline: i64,
    },
    
    /// Cross-chain position rebalancing
    RebalanceRequest {
        user: Pubkey,
        from_protocol_id: u64,
        to_protocol_id: u64,
        amount: u64,
        target_chain_id: u32,
        min_output_amount: u64,
        deadline: i64,
    },
    
    /// Yield rate and protocol updates
    YieldUpdate {
        protocol_id: u64,
        protocol_name: String,
        new_apy: u64,
        tvl: u64,
        available_capacity: u64,
        risk_score: u8,
        last_harvest_time: i64,
        update_timestamp: i64,
    },
    
    /// User position synchronization
    PositionSync {
        user: Pubkey,
        protocol_id: u64,
        principal_amount: u64,
        yield_earned: u64,
        total_shares: u64,
        last_compound_time: i64,
        position_health: u8, // 1-100 scale
        sync_timestamp: i64,
    },
    
    /// Cross-chain yield distribution
    YieldDistribution {
        protocol_id: u64,
        total_yield: u64,
        distribution_rate: u64,
        eligible_users: Vec<Pubkey>,
        per_user_yield: Vec<u64>,
        distribution_timestamp: i64,
    },
    
    /// Emergency pause/unpause across chains
    EmergencyAction {
        action_type: EmergencyActionType,
        protocol_id: Option<u64>,
        reason: String,
        initiated_by: Pubkey,
        timestamp: i64,
    },
    
    /// Cross-chain protocol configuration
    ProtocolConfig {
        protocol_id: u64,
        config_type: ProtocolConfigType,
        config_data: Vec<u8>,
        effective_timestamp: i64,
    },
    
    /// Cross-chain governance voting
    GovernanceVote {
        proposal_id: u64,
        voter: Pubkey,
        vote_weight: u64,
        vote_choice: bool, // true for yes, false for no
        voting_power: u64,
        timestamp: i64,
    },
    
    /// Cross-chain liquidation notice
    LiquidationNotice {
        user: Pubkey,
        protocol_id: u64,
        liquidated_amount: u64,
        liquidation_penalty: u64,
        liquidator: Pubkey,
        timestamp: i64,
    },
    
    /// Cross-chain fee collection
    FeeCollection {
        protocol_id: u64,
        fee_type: FeeType,
        amount: u64,
        token_mint: Pubkey,
        collected_from: Vec<Pubkey>,
        timestamp: i64,
    },
    
    /// Cross-chain oracle price update
    PriceUpdate {
        token_mint: Pubkey,
        price: u64,
        confidence: u64,
        timestamp: i64,
        oracle_source: String,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EmergencyActionType {
    Pause,
    Unpause,
    ForceWithdraw,
    HaltDeposits,
    ResumeDeposits,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ProtocolConfigType {
    YieldRate,
    FeeStructure,
    RiskParameters,
    CapacityLimits,
    RebalanceThresholds,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FeeType {
    Performance,
    Management,
    Withdrawal,
    Rebalance,
    Deposit,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MessageHeader {
    pub msg_type: u8,
    pub version: u32,
    pub message_id: [u8; 32],
    pub payload_length: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CrossChainMessage {
    pub header: MessageHeader,
    pub payload: YieldMessage,
    pub nonce: u64,
    pub timestamp: i64,
    pub signature: Option<Vec<u8>>,
}

impl YieldMessage {
    /// Encode message with full header and metadata
    pub fn encode_with_header(&self, message_id: [u8; 32], nonce: u64) -> Result<Vec<u8>> {
        let payload_bytes = self.try_to_vec()
            .map_err(|_| MsgCodecError::SerializationError)?;
        
        let header = MessageHeader {
            msg_type: self.get_message_type(),
            version: PROTOCOL_VERSION,
            message_id,
            payload_length: payload_bytes.len() as u32,
        };
        
        let cross_chain_msg = CrossChainMessage {
            header,
            payload: self.clone(),
            nonce,
            timestamp: Clock::get()?.unix_timestamp,
            signature: None,
        };
        
        cross_chain_msg.try_to_vec()
            .map_err(|_| MsgCodecError::SerializationError.into())
    }
    
    /// Encode just the payload (backward compatibility)
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.try_to_vec()
            .map_err(|_| MsgCodecError::SerializationError.into())
    }
    
    /// Decode message with header validation
    pub fn decode_with_header(data: &[u8]) -> Result<CrossChainMessage> {
        if data.len() < PAYLOAD_OFFSET {
            return Err(MsgCodecError::InvalidLength.into());
        }
        
        CrossChainMessage::try_from_slice(data)
            .map_err(|_| MsgCodecError::DeserializationError.into())
    }
    
    /// Decode just the payload (backward compatibility)
    pub fn decode(data: &[u8]) -> Result<Self> {
        Self::try_from_slice(data)
            .map_err(|_| MsgCodecError::DeserializationError.into())
    }
    
    /// Get message type identifier
    pub fn get_message_type(&self) -> u8 {
        match self {
            YieldMessage::DepositRequest { .. } => 1,
            YieldMessage::WithdrawRequest { .. } => 2,
            YieldMessage::RebalanceRequest { .. } => 3,
            YieldMessage::YieldUpdate { .. } => 4,
            YieldMessage::PositionSync { .. } => 5,
            YieldMessage::YieldDistribution { .. } => 6,
            YieldMessage::EmergencyAction { .. } => 7,
            YieldMessage::ProtocolConfig { .. } => 8,
            YieldMessage::GovernanceVote { .. } => 9,
            YieldMessage::LiquidationNotice { .. } => 10,
            YieldMessage::FeeCollection { .. } => 11,
            YieldMessage::PriceUpdate { .. } => 12,
        }
    }
    
    /// Validate message based on type and content
    pub fn validate(&self) -> Result<()> {
        match self {
            YieldMessage::DepositRequest { amount, min_apy, deadline, slippage_tolerance, .. } => {
                if *amount == 0 {
                    return Err(ProgramError::InvalidArgument.into());
                }
                if *min_apy > 10000 { // 100% APY max
                    return Err(ProgramError::InvalidArgument.into());
                }
                if *slippage_tolerance > 1000 { // 10% max slippage
                    return Err(ProgramError::InvalidArgument.into());
                }
                if *deadline < Clock::get()?.unix_timestamp {
                    return Err(ProgramError::InvalidArgument.into());
                }
            },
            YieldMessage::WithdrawRequest { amount, deadline, .. } => {
                if *amount == 0 {
                    return Err(ProgramError::InvalidArgument.into());
                }
                if *deadline < Clock::get()?.unix_timestamp {
                    return Err(ProgramError::InvalidArgument.into());
                }
            },
            YieldMessage::RebalanceRequest { amount, min_output_amount, deadline, .. } => {
                if *amount == 0 {
                    return Err(ProgramError::InvalidArgument.into());
                }
                if *min_output_amount > *amount {
                    return Err(ProgramError::InvalidArgument.into());
                }
                if *deadline < Clock::get()?.unix_timestamp {
                    return Err(ProgramError::InvalidArgument.into());
                }
            },
            YieldMessage::YieldUpdate { new_apy, risk_score, .. } => {
                if *new_apy > 10000 { // 100% APY max
                    return Err(ProgramError::InvalidArgument.into());
                }
                if *risk_score > 5 { // Risk score 1-5
                    return Err(ProgramError::InvalidArgument.into());
                }
            },
            YieldMessage::PositionSync { position_health, .. } => {
                if *position_health > 100 {
                    return Err(ProgramError::InvalidArgument.into());
                }
            },
            _ => {}, // Other message types have their own validation
        }
        Ok(())
    }
}

impl CrossChainMessage {
    /// Validate message header
    pub fn validate_header(&self) -> Result<()> {
        if self.header.version != PROTOCOL_VERSION {
            return Err(MsgCodecError::UnsupportedVersion.into());
        }
        
        if self.header.msg_type != self.payload.get_message_type() {
            return Err(MsgCodecError::UnsupportedMessageType.into());
        }
        
        // Validate timestamp (not too old, not too far in future)
        let current_time = Clock::get()?.unix_timestamp;
        if self.timestamp < current_time - 3600 || self.timestamp > current_time + 300 {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        Ok(())
    }
    
    /// Generate message ID from content
    pub fn generate_message_id(&self) -> [u8; 32] {
        use anchor_lang::solana_program::keccak;
        
        let mut hasher_input = Vec::new();
        hasher_input.extend_from_slice(&self.nonce.to_le_bytes());
        hasher_input.extend_from_slice(&self.timestamp.to_le_bytes());
        hasher_input.extend_from_slice(&self.header.msg_type.to_le_bytes());
        
        if let Ok(payload_bytes) = self.payload.try_to_vec() {
            hasher_input.extend_from_slice(&payload_bytes);
        }
        
        keccak::hash(&hasher_input).to_bytes()
    }
}

// =============================================================================
// LEGACY STRING-BASED ENCODING FOR BACKWARD COMPATIBILITY
// =============================================================================

/// Extract the string length from legacy format
fn decode_string_len(buf: &[u8]) -> std::result::Result<usize, MsgCodecError> {
    // Header not long enough
    if buf.len() < STRING_PAYLOAD_OFFSET {
        return Err(MsgCodecError::InvalidLength);
    }
    let mut string_len_bytes = [0u8; 32];
    string_len_bytes.copy_from_slice(&buf[STRING_LENGTH_OFFSET..STRING_LENGTH_OFFSET + 32]);
    // The length is stored in the last 4 bytes (big endian)
    Ok(u32::from_be_bytes(string_len_bytes[28..32].try_into().unwrap()) as usize)
}

/// Legacy string encoding (for backward compatibility with existing LayerZero messages)
pub fn encode(string: &str) -> Vec<u8> {
    let string_bytes = string.as_bytes();
    let mut msg = Vec::with_capacity(
        STRING_PAYLOAD_OFFSET +    // header length
        string_bytes.len()         // string bytes
    );

    // 4 byte length stored at the end of the 32 byte header
    msg.extend(std::iter::repeat(0).take(28)); // padding
    msg.extend_from_slice(&(string_bytes.len() as u32).to_be_bytes());

    // string
    msg.extend_from_slice(string_bytes);

    msg
}

/// Legacy string decoding (for backward compatibility with existing LayerZero messages)
pub fn decode(message: &[u8]) -> std::result::Result<String, MsgCodecError> {
    // Read the declared payload length from the header
    let string_len = decode_string_len(message)?;

    let start: usize = STRING_PAYLOAD_OFFSET;
    // Safely compute end index and check for overflow
    let end = start
        .checked_add(string_len)
        .ok_or(MsgCodecError::InvalidLength)?;

    // Ensure the buffer actually contains the full payload
    if end > message.len() {
        return Err(MsgCodecError::BodyTooShort);
    }

    // Slice out the payload bytes
    let payload = &message[start..end];
    // Attempt to convert the bytes into a Rust string
    match str::from_utf8(payload) {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Err(MsgCodecError::InvalidUtf8),
    }
}

// Utility functions for message routing
pub fn is_user_message(msg_type: u8) -> bool {
    matches!(msg_type, 1 | 2 | 3 | 9) // Deposit, Withdraw, Rebalance, Governance
}

pub fn is_protocol_message(msg_type: u8) -> bool {
    matches!(msg_type, 4 | 6 | 7 | 8 | 11) // YieldUpdate, Distribution, Emergency, Config, Fee
}

pub fn is_system_message(msg_type: u8) -> bool {
    matches!(msg_type, 5 | 10 | 12) // PositionSync, Liquidation, PriceUpdate
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_legacy_string_encoding() {
        let test_string = "Hello, World!";
        let encoded = encode(test_string);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(test_string, decoded);
    }
    
    #[test]
    fn test_message_encoding_decoding() {
        let message = YieldMessage::DepositRequest {
            user: Pubkey::new_unique(),
            amount: 1000000,
            token_mint: Pubkey::new_unique(),
            target_protocol_id: 1,
            min_apy: 500, // 5%
            slippage_tolerance: 100, // 1%
            deadline: 1672531200, // Some future timestamp
            referrer: None,
        };
        
        let encoded = message.encode().unwrap();
        let decoded = YieldMessage::decode(&encoded).unwrap();
        
        assert_eq!(message, decoded);
    }
}