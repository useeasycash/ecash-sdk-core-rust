//! # ecash-sdk-core (Rust)
//!
//! The Official Server-Side SDK for the EasyCash Protocol.
//!
//! `ecash-sdk-core` provides a robust, type-safe Rust interface for interacting
//! with the EasyCash Agent Network. It is designed for high-throughput, secure
//! backend integrations such as:
//!
//! * **Exchanges & Custodians**: Offering private withdrawals for users.
//! * **Payment Gateways**: Processing merchant settlements via stablecoins.
//! * **Payroll Providers**: Batching private salary streams.
//!
//! ## Features
//!
//! * **ZK-Proof Generation**: Built-in logic to generate solvency proofs locally before broadcasting.
//! * **Agentic Routing**: Automatically selects the optimal execution path with multi-factor optimization.
//! * **Built-in Observability**: Metrics tracking for transaction success rates, latency, and fee analysis.
//! * **Performance Optimized**: In-memory caching with TTL for repeated transaction patterns.
//! * **Type Safety**: Strict typing for Assets, Chains, and Intent structures to prevent financial errors.
//! * **Comprehensive Validation**: Input validation for addresses, amounts, and chain compatibility.
//! * **Rate Limiting**: Built-in rate limiting to prevent abuse.
//!
//! ## Quick Start
//!
//! ```no_run
//! use ecash_sdk_core::{EasyCashClient, SdkConfig, TransactionRequest, ChainId, IntentType};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the client
//!     let client = EasyCashClient::new(None)?;
//!
//!     // Create a transaction request
//!     let req = TransactionRequest {
//!         reference_id: "ref_001".to_string(),
//!         intent_type: IntentType::Transfer,
//!         amount: "1000.00".to_string(),
//!         asset: "USDC".to_string(),
//!         recipient: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string()),
//!         source_chain: ChainId::Base,
//!         target_chain: None,
//!         is_shielded: true,
//!     };
//!
//!     // Execute the transaction
//!     let response = client.execute_transaction(&req).await?;
//!     println!("Transaction hash: {}", response.tx_hash);
//!
//!     Ok(())
//! }
//! ```

pub mod agent;
pub mod cache;
pub mod client;
pub mod config;
pub mod crypto;
pub mod errors;
pub mod monitoring;
pub mod rate_limiter;
pub mod types;
pub mod validator;
pub mod zk;

// Re-export main types for convenience
pub use client::EasyCashClient;
pub use config::SdkConfig;
pub use errors::{ErrorCode, Result, SdkError};
pub use types::{ChainId, IntentType, TransactionRequest, TransactionResponse};

// Re-export commonly used traits
pub use agent::AgentNegotiatorTrait;
pub use zk::ZkProofGenerator;
