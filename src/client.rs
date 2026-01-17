use crate::agent::{AgentNegotiator, AgentNegotiatorTrait};
use crate::cache::Cache;
use crate::config::SdkConfig;
use crate::errors::{ErrorCode, Result, SdkError};
use crate::monitoring::Metrics;
use crate::types::{TransactionRequest, TransactionResponse};
use crate::validator;
use crate::zk::{ProofGenerator, ZkProofGenerator};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Main entry point for the SDK
pub struct EasyCashClient {
    config: SdkConfig,
    zk: ProofGenerator,
    negotiator: AgentNegotiator,
    cache: Option<Cache<TransactionResponse>>,
    metrics: Metrics,
}

impl EasyCashClient {
    /// Initializes a new EasyCash SDK client with full configuration
    pub fn new(config: Option<SdkConfig>) -> Result<Self> {
        let cfg = config.unwrap_or_else(SdkConfig::default_config);

        cfg.validate()
            .map_err(|e| SdkError::new(ErrorCode::InvalidRequest, format!("invalid configuration: {}", e)))?;

        let mut client = Self {
            config: cfg.clone(),
            zk: ProofGenerator::new("./circuits/spend.wasm"),
            negotiator: AgentNegotiator::new(cfg.timeout),
            cache: None,
            metrics: Metrics::new(),
        };

        if cfg.enable_caching {
            client.cache = Some(Cache::new(cfg.cache_ttl));
        }

        Ok(client)
    }

    /// Constructs a transfer intent and executes it with full validation
    pub async fn execute_transaction(
        &self,
        req: &TransactionRequest,
    ) -> Result<TransactionResponse> {
        let start_time = Instant::now();
        
        // Execute transaction and capture result
        let result = self.execute_transaction_internal(req).await;
        
        // Record metrics based on actual result
        if self.config.enable_metrics {
            let success = result.is_ok();
            let fee = result.as_ref().map(|r| {
                // Try to parse fee from response, default to 0.0
                r.fee_used.split_whitespace().next()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0)
            }).unwrap_or(0.0);
            let latency = start_time.elapsed();
            
            self.metrics.record_transaction(success, fee, latency);
        }
        
        result
    }

    async fn execute_transaction_internal(
        &self,
        req: &TransactionRequest,
    ) -> Result<TransactionResponse> {

        // 1. Validate Request
        validator::validate_transaction_request(req)
            .map_err(|e| SdkError::new(ErrorCode::InvalidRequest, format!("validation failed: {}", e)))?;

        // 2. Check Cache for similar recent transactions
        if let Some(ref cache) = self.cache {
            let cache_key = format!("{}-{}-{}", req.intent_type.as_str(), req.amount, req.asset);
            if let Some(cached) = cache.get(&cache_key) {
                tracing::info!("[SDK] Cache hit for transaction pattern");
                return Ok(cached);
            }
        }

        // 3. Generate ZK Proof if shielded
        if self.config.enable_zk_proofs && req.is_shielded {
            let proof = self
                .zk
                .generate_solvency_proof(&req.amount, "0")
                .map_err(|e| SdkError::new(ErrorCode::ProofGeneration, format!("failed to generate privacy proof: {}", e)))?;
            tracing::info!("[SDK] Generated ZK Proof: {}...", &proof[..10.min(proof.len())]);
        }

        // 4. Request quotes from agents
        let quotes = self
            .negotiator
            .request_quotes(req)
            .await
            .map_err(|e| SdkError::new(ErrorCode::AgentUnavailable, format!("failed to get agent quotes: {}", e)))?;

        // 5. Select best route
        let best_route = self
            .negotiator
            .select_best_route(&quotes, "balanced")
            .map_err(|e| SdkError::new(ErrorCode::AgentUnavailable, format!("no suitable route found: {}", e)))?;

        tracing::info!(
            "[SDK] Selected Agent: {} (Fee: {}, Security: {:.2})",
            best_route.agent_id,
            best_route.estimated_fee,
            best_route.security_score
        );

        // 6. Execute via selected agent
        // NOTE: This is a mock execution. Real implementation would:
        // - Submit transaction to selected agent
        // - Wait for on-chain confirmation
        // - Handle retries and error cases
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 7. Construct Response
        // NOTE: In production, tx_hash and block_height come from blockchain
        let tx_hash = format!("0x{}", Uuid::new_v4().to_string().replace("-", ""));

        let resp = TransactionResponse {
            tx_hash,
            status: "confirmed".to_string(),
            block_height: 1948201,
            fee_used: best_route.estimated_fee.clone(),
        };

        // 8. Cache successful result
        if let Some(ref cache) = self.cache {
            let cache_key = format!("{}-{}-{}", req.intent_type.as_str(), req.amount, req.asset);
            cache.set(cache_key, resp.clone());
        }

        Ok(resp)
    }

    /// Returns current SDK performance metrics
    pub fn get_metrics(&self) -> std::collections::HashMap<String, f64> {
        if !self.config.enable_metrics {
            let mut map = std::collections::HashMap::new();
            map.insert("metrics_disabled".to_string(), 1.0);
            return map;
        }
        self.metrics.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChainId, IntentType, TransactionRequest};

    #[tokio::test]
    async fn test_client_new() {
        let config = SdkConfig::default_config();
        let client = EasyCashClient::new(Some(config));
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_client_new_default() {
        let client = EasyCashClient::new(None);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_execute_transaction() {
        let client = EasyCashClient::new(None).unwrap();
        let req = TransactionRequest {
            reference_id: "ref_001".to_string(),
            intent_type: IntentType::Transfer,
            amount: "1000.00".to_string(),
            asset: "USDC".to_string(),
            recipient: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string()),
            source_chain: ChainId::Base,
            target_chain: None,
            is_shielded: false,
        };

        let resp = client.execute_transaction(&req).await;
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        assert!(resp.tx_hash.starts_with("0x"));
        assert_eq!(resp.status, "confirmed");
    }

    #[tokio::test]
    async fn test_execute_transaction_with_shield() {
        let client = EasyCashClient::new(None).unwrap();
        let req = TransactionRequest {
            reference_id: "ref_002".to_string(),
            intent_type: IntentType::Transfer,
            amount: "1000.00".to_string(),
            asset: "USDC".to_string(),
            recipient: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string()),
            source_chain: ChainId::Base,
            target_chain: None,
            is_shielded: true,
        };

        let resp = client.execute_transaction(&req).await;
        assert!(resp.is_ok());
    }

    #[tokio::test]
    async fn test_execute_transaction_invalid() {
        let client = EasyCashClient::new(None).unwrap();
        let req = TransactionRequest {
            reference_id: "ref_003".to_string(),
            intent_type: IntentType::Transfer,
            amount: "".to_string(), // Invalid: empty amount
            asset: "USDC".to_string(),
            recipient: None,
            source_chain: ChainId::Base,
            target_chain: None,
            is_shielded: false,
        };

        let resp = client.execute_transaction(&req).await;
        assert!(resp.is_err());
    }

    #[tokio::test]
    async fn test_execute_transaction_caching() {
        let mut config = SdkConfig::default_config();
        config.enable_caching = true;
        let client = EasyCashClient::new(Some(config)).unwrap();
        
        let req = TransactionRequest {
            reference_id: "ref_004".to_string(),
            intent_type: IntentType::Transfer,
            amount: "1000.00".to_string(),
            asset: "USDC".to_string(),
            recipient: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string()),
            source_chain: ChainId::Base,
            target_chain: None,
            is_shielded: false,
        };

        // First call
        let resp1 = client.execute_transaction(&req).await.unwrap();
        
        // Second call should hit cache
        let resp2 = client.execute_transaction(&req).await.unwrap();
        assert_eq!(resp1.tx_hash, resp2.tx_hash);
    }

    #[tokio::test]
    async fn test_get_metrics() {
        let client = EasyCashClient::new(None).unwrap();
        let metrics = client.get_metrics();
        assert!(metrics.contains_key("total_transactions"));
    }
}
