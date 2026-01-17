use crate::types::TransactionRequest;
use std::time::Duration;

/// Route quote from an agent for executing a transaction.
///
/// Contains all information needed to evaluate and execute a transaction route
/// through the EasyCash agent network.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteQuote {
    /// Unique identifier for the agent providing this quote
    pub agent_id: String,
    /// Estimated fee in human-readable format (e.g., "0.05 USDC")
    pub estimated_fee: String,
    /// Estimated time to complete the transaction
    pub estimated_time: Duration,
    /// Chain hops for the route (e.g., ["base", "polygon", "ethereum"])
    pub route: Vec<String>,
    /// Security score from 0.0 (lowest) to 1.0 (highest)
    pub security_score: f64,
}

/// Trait for agent negotiation (allows for future real implementation).
///
/// This trait defines the interface for requesting quotes from agents
/// and selecting the optimal route for transaction execution.
#[async_trait::async_trait]
pub trait AgentNegotiatorTrait: Send + Sync {
    /// Fetches multiple route quotes from available agents.
    ///
    /// # Arguments
    /// * `req` - The transaction request to get quotes for
    ///
    /// # Returns
    /// * `Ok(Vec<RouteQuote>)` - List of quotes from available agents
    /// * `Err(String)` - Error message if quote fetching fails
    async fn request_quotes(&self, req: &TransactionRequest) -> Result<Vec<RouteQuote>, String>;

    /// Applies multi-factor optimization to choose the best agent.
    ///
    /// # Arguments
    /// * `quotes` - Slice of available route quotes
    /// * `preference` - Optimization preference ("speed", "cost", "security", "balanced")
    ///
    /// # Returns
    /// * `Ok(RouteQuote)` - The selected best route (cloned)
    /// * `Err(String)` - Error if no suitable route found
    fn select_best_route(
        &self,
        quotes: &[RouteQuote],
        preference: &str,
    ) -> Result<RouteQuote, String>;
}

/// Mock agent negotiator for development/testing.
///
/// **NOTE: This is a simulation/mock implementation.**
/// In production, this should make real network calls to the Agent Discovery Service
/// and negotiate with actual agent nodes in the EasyCash network.
///
/// # Example
/// ```
/// use ecash_sdk_core::agent::MockAgentNegotiator;
/// use std::time::Duration;
///
/// let negotiator = MockAgentNegotiator::new(Duration::from_secs(30));
/// ```
pub struct MockAgentNegotiator {
    /// Timeout for agent negotiation requests
    #[allow(dead_code)]
    timeout: Duration,
}

impl MockAgentNegotiator {
    /// Creates a new mock agent negotiator with the specified timeout.
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for agent responses
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Returns the configured timeout duration.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[async_trait::async_trait]
impl AgentNegotiatorTrait for MockAgentNegotiator {
    /// Fetches multiple route quotes from available agents.
    /// 
    /// **MOCK IMPLEMENTATION**: Returns hardcoded quotes.
    /// Real implementation would:
    /// 1. Query Agent Discovery Service
    /// 2. Request quotes from multiple agents
    /// 3. Validate agent reputation and stake
    async fn request_quotes(
        &self,
        req: &TransactionRequest,
    ) -> Result<Vec<RouteQuote>, String> {
        // Simulate network call latency
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Return simulated quotes
        let quotes = vec![
            RouteQuote {
                agent_id: "agent-001".to_string(),
                estimated_fee: "0.05 USDC".to_string(),
                estimated_time: Duration::from_secs(15),
                route: vec![
                    req.source_chain.as_str().to_string(),
                    req.target_chain
                        .map(|c| c.as_str().to_string())
                        .unwrap_or_else(|| req.source_chain.as_str().to_string()),
                ],
                security_score: 0.98,
            },
            RouteQuote {
                agent_id: "agent-002".to_string(),
                estimated_fee: "0.03 USDC".to_string(),
                estimated_time: Duration::from_secs(30),
                route: vec![
                    req.source_chain.as_str().to_string(),
                    "polygon".to_string(),
                    req.target_chain
                        .map(|c| c.as_str().to_string())
                        .unwrap_or_else(|| req.source_chain.as_str().to_string()),
                ],
                security_score: 0.85,
            },
        ];

        Ok(quotes)
    }

    /// Applies multi-factor optimization to choose the best agent.
    ///
    /// Supports multiple preference modes:
    /// - "speed": Prioritize fastest execution time
    /// - "cost": Prioritize lowest fees
    /// - "security": Prioritize highest security score
    /// - "balanced" (default): Weighted combination of all factors
    fn select_best_route(
        &self,
        quotes: &[RouteQuote],
        preference: &str,
    ) -> Result<RouteQuote, String> {
        if quotes.is_empty() {
            return Err("no quotes available".to_string());
        }

        let best = match preference {
            "speed" => quotes
                .iter()
                .min_by(|a, b| a.estimated_time.cmp(&b.estimated_time)),
            "cost" => {
                // Parse fee and find minimum (assumes format "X.XX ASSET")
                quotes.iter().min_by(|a, b| {
                    let fee_a: f64 = a
                        .estimated_fee
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(f64::MAX);
                    let fee_b: f64 = b
                        .estimated_fee
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(f64::MAX);
                    fee_a.partial_cmp(&fee_b).unwrap_or(std::cmp::Ordering::Equal)
                })
            }
            "security" => quotes.iter().max_by(|a, b| {
                a.security_score
                    .partial_cmp(&b.security_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            _ => {
                // "balanced" - weighted score (security has higher weight)
                quotes.iter().max_by(|a, b| {
                    let score_a = a.security_score * 0.5
                        + (1.0 / (a.estimated_time.as_secs_f64() + 1.0)) * 0.3
                        + (1.0
                            / (a.estimated_fee
                                .split_whitespace()
                                .next()
                                .and_then(|s| s.parse::<f64>().ok())
                                .unwrap_or(1.0)
                                + 1.0))
                            * 0.2;
                    let score_b = b.security_score * 0.5
                        + (1.0 / (b.estimated_time.as_secs_f64() + 1.0)) * 0.3
                        + (1.0
                            / (b.estimated_fee
                                .split_whitespace()
                                .next()
                                .and_then(|s| s.parse::<f64>().ok())
                                .unwrap_or(1.0)
                                + 1.0))
                            * 0.2;
                    score_a
                        .partial_cmp(&score_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            }
        };

        best.cloned()
            .ok_or_else(|| "no quotes available".to_string())
    }
}

/// Type alias for current agent negotiator (can be swapped for real implementation)
pub type AgentNegotiator = MockAgentNegotiator;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChainId, IntentType, TransactionRequest};

    #[tokio::test]
    async fn test_request_quotes() {
        let negotiator = MockAgentNegotiator::new(Duration::from_secs(30));
        let req = TransactionRequest {
            reference_id: "ref_001".to_string(),
            intent_type: IntentType::Transfer,
            amount: "1000.00".to_string(),
            asset: "USDC".to_string(),
            recipient: None,
            source_chain: ChainId::Base,
            target_chain: Some(ChainId::Ethereum),
            is_shielded: false,
        };

        let quotes = negotiator.request_quotes(&req).await.unwrap();
        assert_eq!(quotes.len(), 2);
        assert_eq!(quotes[0].agent_id, "agent-001");
        assert_eq!(quotes[1].agent_id, "agent-002");
    }

    #[test]
    fn test_select_best_route_balanced() {
        let negotiator = MockAgentNegotiator::new(Duration::from_secs(30));
        let quotes = vec![
            RouteQuote {
                agent_id: "agent-001".to_string(),
                estimated_fee: "0.05 USDC".to_string(),
                estimated_time: Duration::from_secs(15),
                route: vec!["base".to_string(), "ethereum".to_string()],
                security_score: 0.98,
            },
            RouteQuote {
                agent_id: "agent-002".to_string(),
                estimated_fee: "0.03 USDC".to_string(),
                estimated_time: Duration::from_secs(30),
                route: vec!["base".to_string(), "polygon".to_string(), "ethereum".to_string()],
                security_score: 0.85,
            },
        ];

        let best = negotiator.select_best_route(&quotes, "balanced").unwrap();
        assert_eq!(best.agent_id, "agent-001");
        assert_eq!(best.security_score, 0.98);
    }

    #[test]
    fn test_select_best_route_by_cost() {
        let negotiator = MockAgentNegotiator::new(Duration::from_secs(30));
        let quotes = vec![
            RouteQuote {
                agent_id: "agent-001".to_string(),
                estimated_fee: "0.05 USDC".to_string(),
                estimated_time: Duration::from_secs(15),
                route: vec!["base".to_string()],
                security_score: 0.98,
            },
            RouteQuote {
                agent_id: "agent-002".to_string(),
                estimated_fee: "0.02 USDC".to_string(),
                estimated_time: Duration::from_secs(30),
                route: vec!["base".to_string()],
                security_score: 0.85,
            },
        ];

        let best = negotiator.select_best_route(&quotes, "cost").unwrap();
        assert_eq!(best.agent_id, "agent-002");
        assert_eq!(best.estimated_fee, "0.02 USDC");
    }

    #[test]
    fn test_select_best_route_by_speed() {
        let negotiator = MockAgentNegotiator::new(Duration::from_secs(30));
        let quotes = vec![
            RouteQuote {
                agent_id: "agent-001".to_string(),
                estimated_fee: "0.05 USDC".to_string(),
                estimated_time: Duration::from_secs(60),
                route: vec!["base".to_string()],
                security_score: 0.98,
            },
            RouteQuote {
                agent_id: "agent-002".to_string(),
                estimated_fee: "0.10 USDC".to_string(),
                estimated_time: Duration::from_secs(10),
                route: vec!["base".to_string()],
                security_score: 0.85,
            },
        ];

        let best = negotiator.select_best_route(&quotes, "speed").unwrap();
        assert_eq!(best.agent_id, "agent-002");
        assert_eq!(best.estimated_time, Duration::from_secs(10));
    }

    #[test]
    fn test_select_best_route_by_security() {
        let negotiator = MockAgentNegotiator::new(Duration::from_secs(30));
        let quotes = vec![
            RouteQuote {
                agent_id: "agent-001".to_string(),
                estimated_fee: "0.01 USDC".to_string(),
                estimated_time: Duration::from_secs(5),
                route: vec!["base".to_string()],
                security_score: 0.70,
            },
            RouteQuote {
                agent_id: "agent-002".to_string(),
                estimated_fee: "0.10 USDC".to_string(),
                estimated_time: Duration::from_secs(60),
                route: vec!["base".to_string()],
                security_score: 0.99,
            },
        ];

        let best = negotiator.select_best_route(&quotes, "security").unwrap();
        assert_eq!(best.agent_id, "agent-002");
        assert_eq!(best.security_score, 0.99);
    }

    #[test]
    fn test_select_best_route_empty() {
        let negotiator = MockAgentNegotiator::new(Duration::from_secs(30));
        let quotes: Vec<RouteQuote> = vec![];
        assert!(negotiator.select_best_route(&quotes, "balanced").is_err());
    }
}
