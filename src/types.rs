use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainId {
    Ethereum,
    Base,
    Solana,
}

impl ChainId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChainId::Ethereum => "ethereum",
            ChainId::Base => "base",
            ChainId::Solana => "solana",
        }
    }
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ChainId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ethereum" => Ok(ChainId::Ethereum),
            "base" => Ok(ChainId::Base),
            "solana" => Ok(ChainId::Solana),
            _ => Err(format!("unknown chain: {}", s)),
        }
    }
}

/// Classification of the operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IntentType {
    Transfer,
    Swap,
    Shield,
}

impl IntentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntentType::Transfer => "transfer",
            IntentType::Swap => "swap",
            IntentType::Shield => "shield",
        }
    }
}

impl std::fmt::Display for IntentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for IntentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "transfer" => Ok(IntentType::Transfer),
            "swap" => Ok(IntentType::Swap),
            "shield" => Ok(IntentType::Shield),
            _ => Err(format!("unknown intent type: {}", s)),
        }
    }
}

/// Standard payload for initiating an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    #[serde(rename = "reference_id")]
    pub reference_id: String,
    #[serde(rename = "type")]
    pub intent_type: IntentType,
    pub amount: String, // String to support big integers
    pub asset: String,  // e.g., "USDC"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    #[serde(rename = "source_chain")]
    pub source_chain: ChainId,
    #[serde(rename = "target_chain", skip_serializing_if = "Option::is_none")]
    pub target_chain: Option<ChainId>,
    /// Privacy options
    #[serde(rename = "is_shielded")]
    pub is_shielded: bool,
}

impl TransactionRequest {
    /// Validates the transaction request
    pub fn validate(&self) -> Result<(), String> {
        if self.amount.is_empty() {
            return Err("amount is required".to_string());
        }
        if self.asset.is_empty() {
            return Err("asset is required".to_string());
        }
        Ok(())
    }
}

/// Result of an intent execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionResponse {
    #[serde(rename = "tx_hash")]
    pub tx_hash: String,
    pub status: String,
    #[serde(rename = "block_height")]
    pub block_height: u64,
    #[serde(rename = "fee_used")]
    pub fee_used: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_display() {
        assert_eq!(ChainId::Ethereum.to_string(), "ethereum");
        assert_eq!(ChainId::Base.to_string(), "base");
        assert_eq!(ChainId::Solana.to_string(), "solana");
    }

    #[test]
    fn test_chain_id_as_str() {
        assert_eq!(ChainId::Ethereum.as_str(), "ethereum");
        assert_eq!(ChainId::Base.as_str(), "base");
        assert_eq!(ChainId::Solana.as_str(), "solana");
    }

    #[test]
    fn test_intent_type_display() {
        assert_eq!(IntentType::Transfer.to_string(), "transfer");
        assert_eq!(IntentType::Swap.to_string(), "swap");
        assert_eq!(IntentType::Shield.to_string(), "shield");
    }

    #[test]
    fn test_transaction_request_validate() {
        let req = TransactionRequest {
            reference_id: "ref_001".to_string(),
            intent_type: IntentType::Transfer,
            amount: "1000.00".to_string(),
            asset: "USDC".to_string(),
            recipient: None,
            source_chain: ChainId::Base,
            target_chain: None,
            is_shielded: false,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_transaction_request_validate_empty_amount() {
        let req = TransactionRequest {
            reference_id: "ref_001".to_string(),
            intent_type: IntentType::Transfer,
            amount: "".to_string(),
            asset: "USDC".to_string(),
            recipient: None,
            source_chain: ChainId::Base,
            target_chain: None,
            is_shielded: false,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_transaction_request_serialize() {
        let req = TransactionRequest {
            reference_id: "ref_001".to_string(),
            intent_type: IntentType::Transfer,
            amount: "1000.00".to_string(),
            asset: "USDC".to_string(),
            recipient: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string()),
            source_chain: ChainId::Base,
            target_chain: Some(ChainId::Ethereum),
            is_shielded: true,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("transfer"));
        assert!(json.contains("base"));
        assert!(json.contains("is_shielded"));
    }

    #[test]
    fn test_chain_id_from_str() {
        assert_eq!(ChainId::from_str("ethereum").unwrap(), ChainId::Ethereum);
        assert_eq!(ChainId::from_str("base").unwrap(), ChainId::Base);
        assert_eq!(ChainId::from_str("solana").unwrap(), ChainId::Solana);
        assert!(ChainId::from_str("invalid").is_err());
    }

    #[test]
    fn test_intent_type_from_str() {
        assert_eq!(IntentType::from_str("transfer").unwrap(), IntentType::Transfer);
        assert_eq!(IntentType::from_str("swap").unwrap(), IntentType::Swap);
        assert_eq!(IntentType::from_str("shield").unwrap(), IntentType::Shield);
        assert!(IntentType::from_str("invalid").is_err());
    }
}
