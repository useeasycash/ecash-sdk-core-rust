use regex::Regex;
use crate::types::{ChainId, TransactionRequest};

lazy_static::lazy_static! {
    static ref ADDRESS_REGEX: Regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
    static ref AMOUNT_REGEX: Regex = Regex::new(r"^\d+(\.\d+)?$").unwrap();
}

/// Validates an Ethereum address format
pub fn validate_address(address: &str) -> Result<(), String> {
    if !ADDRESS_REGEX.is_match(address) {
        return Err(format!("invalid address format: {}", address));
    }
    Ok(())
}

/// Validates an amount string
pub fn validate_amount(amount: &str) -> Result<(), String> {
    if amount.is_empty() {
        return Err("amount cannot be empty".to_string());
    }
    
    if !AMOUNT_REGEX.is_match(amount) {
        return Err(format!("invalid amount format: {} (expected positive number)", amount));
    }

    // Check if amount is positive
    let val: f64 = amount.parse().map_err(|e| format!("failed to parse amount: {}", e))?;
    if val <= 0.0 {
        return Err("amount must be positive".to_string());
    }
    
    // Check for reasonable upper bound (prevent overflow issues)
    if val > 1e15 {
        return Err("amount exceeds maximum allowed value".to_string());
    }

    Ok(())
}

/// Validates if a chain ID is supported
pub fn validate_chain(chain: ChainId) -> Result<(), String> {
    // All defined ChainId variants are valid
    match chain {
        ChainId::Ethereum | ChainId::Base | ChainId::Solana => Ok(()),
    }
}

/// Performs comprehensive validation on a transaction request
pub fn validate_transaction_request(req: &TransactionRequest) -> Result<(), String> {
    validate_amount(&req.amount)
        .map_err(|e| format!("amount validation failed: {}", e))?;

    validate_chain(req.source_chain)
        .map_err(|e| format!("source chain validation failed: {}", e))?;

    if let Some(target_chain) = req.target_chain {
        validate_chain(target_chain)
            .map_err(|e| format!("target chain validation failed: {}", e))?;
    }

    if let Some(ref recipient) = req.recipient {
        validate_address(recipient)
            .map_err(|e| format!("recipient validation failed: {}", e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChainId, IntentType, TransactionRequest};

    #[test]
    fn test_validate_address_valid() {
        assert!(validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0").is_ok());
    }

    #[test]
    fn test_validate_address_invalid_short() {
        assert!(validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bE").is_err());
    }

    #[test]
    fn test_validate_address_invalid_no_prefix() {
        assert!(validate_address("742d35Cc6634C0532925a3b844Bc9e7595f0bEb").is_err());
    }

    #[test]
    fn test_validate_amount_valid() {
        assert!(validate_amount("100.50").is_ok());
        assert!(validate_amount("1000").is_ok());
        assert!(validate_amount("0.001").is_ok());
    }

    #[test]
    fn test_validate_amount_invalid_negative() {
        assert!(validate_amount("-100").is_err());
    }

    #[test]
    fn test_validate_amount_invalid_zero() {
        assert!(validate_amount("0").is_err());
    }

    #[test]
    fn test_validate_amount_invalid_format() {
        assert!(validate_amount("abc").is_err());
        assert!(validate_amount("100.50.25").is_err());
        assert!(validate_amount("").is_err());
    }

    #[test]
    fn test_validate_amount_too_large() {
        assert!(validate_amount("2000000000000000").is_err()); // > 1e15 (2e15)
    }

    #[test]
    fn test_validate_chain() {
        assert!(validate_chain(ChainId::Ethereum).is_ok());
        assert!(validate_chain(ChainId::Base).is_ok());
        assert!(validate_chain(ChainId::Solana).is_ok());
    }

    #[test]
    fn test_validate_transaction_request_valid() {
        let req = TransactionRequest {
            reference_id: "ref_001".to_string(),
            intent_type: IntentType::Transfer,
            amount: "1000.00".to_string(),
            asset: "USDC".to_string(),
            recipient: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string()),
            source_chain: ChainId::Base,
            target_chain: None,
            is_shielded: true,
        };
        assert!(validate_transaction_request(&req).is_ok());
    }

    #[test]
    fn test_validate_transaction_request_invalid_amount() {
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
        assert!(validate_transaction_request(&req).is_err());
    }

    #[test]
    fn test_validate_transaction_request_invalid_recipient() {
        let req = TransactionRequest {
            reference_id: "ref_001".to_string(),
            intent_type: IntentType::Transfer,
            amount: "1000.00".to_string(),
            asset: "USDC".to_string(),
            recipient: Some("invalid_address".to_string()),
            source_chain: ChainId::Base,
            target_chain: None,
            is_shielded: false,
        };
        assert!(validate_transaction_request(&req).is_err());
    }
}
