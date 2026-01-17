# ecash-sdk-core (Rust)

[![Crates.io](https://img.shields.io/crates/v/ecash-sdk-core.svg)](https://crates.io/crates/ecash-sdk-core)
[![Documentation](https://docs.rs/ecash-sdk-core/badge.svg)](https://docs.rs/ecash-sdk-core)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**The Official Server-Side SDK for the EasyCash Protocol (Rust Edition).**

`ecash-sdk-core` provides a robust, type-safe Rust interface for interacting with the EasyCash Agent Network. It is designed for high-throughput, secure backend integrations such as:

*   **Exchanges & Custodians**: Offering private withdrawals for users.
*   **Payment Gateways**: Processing merchant settlements via stablecoins.
*   **Payroll Providers**: Batching private salary streams.

## 🚀 Key Features

*   **🔒 ZK-Proof Generation**: Built-in logic to generate solvency proofs locally before broadcasting.
*   **🤖 Agentic Routing**: Automatically selects the optimal execution path with multi-factor optimization (cost, speed, security).
*   **📊 Built-in Observability**: Metrics tracking for transaction success rates, latency, and fee analysis.
*   **⚡ Performance Optimized**: In-memory caching with TTL for repeated transaction patterns.
*   **🛡 Type Safety**: Strict typing for Assets, Chains, and Intent structures to prevent financial errors.
*   **✅ Comprehensive Validation**: Input validation for addresses, amounts, and chain compatibility.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ecash-sdk-core = "0.1.0"
tokio = { version = "1.35", features = ["full"] }
```

## 🛠 Quick Start

### Basic Usage

```rust
use ecash_sdk_core::{EasyCashClient, SdkConfig, IntentType, ChainId, TransactionRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with default config
    let cfg = SdkConfig::default_config()
        .with_api_key("your_api_key_here");
    
    let sdk = EasyCashClient::new(Some(cfg))?;

    // Execute a private transfer
    let req = TransactionRequest {
        reference_id: "ref_001".to_string(),
        intent_type: IntentType::Transfer,
        amount: "1000.00".to_string(),
        asset: "USDC".to_string(),
        recipient: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string()),
        source_chain: ChainId::Base,
        target_chain: None,
        is_shielded: true, // Enable ZK Privacy
    };

    let resp = sdk.execute_transaction(&req).await?;

    println!("Transaction confirmed: {}", resp.tx_hash);
    Ok(())
}
```

### Advanced Configuration

```rust
use std::time::Duration;
use ecash_sdk_core::SdkConfig;

let cfg = SdkConfig {
    api_endpoint: std::env::var("ECASH_API_ENDPOINT")
        .unwrap_or_else(|_| "https://api.useeasy.cash".to_string()),
    api_key: std::env::var("ECASH_API_KEY").unwrap_or_default(),
    environment: "mainnet".to_string(),
    timeout: Duration::from_secs(30),
    max_retries: 3,
    retry_backoff: Duration::from_secs(2),
    enable_zk_proofs: true,
    proof_cache_ttl: Duration::from_secs(300),
    enable_metrics: true,
    enable_caching: true,
    cache_ttl: Duration::from_secs(60),
};

let sdk = EasyCashClient::new(Some(cfg))?;
```

### Monitoring & Metrics

```rust
// Get performance metrics
let metrics = sdk.get_metrics();
println!("Success Rate: {:.2}%", metrics["success_rate"] * 100.0);
println!("Average Latency: {}ms", metrics["average_latency_ms"]);
```

## 🏗 Architecture

```
src/
├── agent/          # Route negotiation & quote selection
├── cache/          # In-memory caching with TTL
├── client/         # Main SDK client interface
├── config/         # Configuration management
├── crypto/         # Cryptographic signing utilities
├── errors/         # Structured error handling
├── monitoring/     # Metrics & observability
├── types/          # Domain models & types
├── validator/      # Input validation
└── zk/             # Zero-Knowledge proof generation
```

### Package Overview

*   **`client`**: High-level facade for API interaction with automatic route optimization.
*   **`agent`**: Negotiates with the decentralized agent network to find optimal execution paths.
*   **`zk`**: Zero-Knowledge cryptographic primitives and proof generation logic.
*   **`monitoring`**: Real-time metrics collection for performance analysis.
*   **`cache`**: Performance optimization through intelligent caching.
*   **`validator`**: Comprehensive input validation to prevent errors.
*   **`config`**: Environment-aware configuration with sensible defaults.
*   **`errors`**: Structured error codes for better error handling.

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run a specific test:

```bash
cargo test --package ecash-sdk-core --lib validator::tests
```

## 📖 Examples

Check out the `examples/` directory for complete working examples:

```bash
cargo run --example simple_transfer
```

## 🔧 Development

```bash
# Install dependencies
cargo build

# Run linter
cargo clippy

# Run tests
cargo test

# Run example
cargo run --example simple_transfer
```

## 🌐 Environment Variables

```bash
ECASH_API_KEY=your_api_key_here
ECASH_API_ENDPOINT=https://api.useeasy.cash
ECASH_ENV=mainnet  # or testnet, devnet
```

## 🤝 Contributing

We welcome Pull Requests! Please ensure you run `cargo test` and `cargo clippy` before submitting.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.

## 🔗 Links

*   [EasyCash Website](https://useeasy.cash)
*   [Documentation](https://useeasycash.gitbook.io/ecash-docs/)
*   [Twitter](https://x.com/useeasycash)
*   [GitHub Organization](https://github.com/useeasycash)
