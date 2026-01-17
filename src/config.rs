use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Global configuration for the SDK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    /// API Configuration
    #[serde(rename = "api_endpoint")]
    pub api_endpoint: String,
    #[serde(rename = "api_key")]
    pub api_key: String,
    pub environment: String, // "mainnet" | "testnet" | "devnet"

    /// Network Configuration
    pub timeout: Duration,
    #[serde(rename = "max_retries")]
    pub max_retries: u32,
    #[serde(rename = "retry_backoff")]
    pub retry_backoff: Duration,

    /// Privacy Configuration
    #[serde(rename = "enable_zk_proofs")]
    pub enable_zk_proofs: bool,
    #[serde(rename = "proof_cache_ttl")]
    pub proof_cache_ttl: Duration,

    /// Performance Configuration
    #[serde(rename = "enable_metrics")]
    pub enable_metrics: bool,
    #[serde(rename = "enable_caching")]
    pub enable_caching: bool,
    #[serde(rename = "cache_ttl")]
    pub cache_ttl: Duration,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            api_endpoint: std::env::var("ECASH_API_ENDPOINT")
                .unwrap_or_else(|_| "https://api.useeasy.cash".to_string()),
            api_key: std::env::var("ECASH_API_KEY").unwrap_or_default(),
            environment: std::env::var("ECASH_ENV")
                .unwrap_or_else(|_| "mainnet".to_string()),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_backoff: Duration::from_secs(2),
            enable_zk_proofs: true,
            proof_cache_ttl: Duration::from_secs(300), // 5 minutes
            enable_metrics: true,
            enable_caching: true,
            cache_ttl: Duration::from_secs(60), // 1 minute
        }
    }
}

impl SdkConfig {
    /// Returns sensible defaults
    pub fn default_config() -> Self {
        Self::default()
    }
    
    /// Sets the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.timeout.as_secs() == 0 {
            return Err("timeout must be greater than 0".to_string());
        }
        if self.cache_ttl.as_secs() == 0 {
            return Err("cache_ttl must be greater than 0".to_string());
        }
        if self.proof_cache_ttl.as_secs() == 0 {
            return Err("proof_cache_ttl must be greater than 0".to_string());
        }
        if !matches!(self.environment.as_str(), "mainnet" | "testnet" | "devnet") {
            return Err(format!("invalid environment: {} (must be mainnet, testnet, or devnet)", self.environment));
        }
        if self.max_retries == 0 {
            return Err("max_retries must be greater than 0".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SdkConfig::default_config();
        assert!(!config.api_endpoint.is_empty());
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_zk_proofs);
        assert!(config.enable_metrics);
    }

    #[test]
    fn test_config_with_api_key() {
        let config = SdkConfig::default_config().with_api_key("test_key");
        assert_eq!(config.api_key, "test_key");
    }

    #[test]
    fn test_config_validate() {
        let mut config = SdkConfig::default_config();
        assert!(config.validate().is_ok());
        
        config.timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_environment() {
        let mut config = SdkConfig::default_config();
        config.environment = "invalid".to_string();
        assert!(config.validate().is_err());
        
        config.environment = "mainnet".to_string();
        assert!(config.validate().is_ok());
        
        config.environment = "testnet".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_max_retries() {
        let mut config = SdkConfig::default_config();
        config.max_retries = 0;
        assert!(config.validate().is_err());
    }
}
