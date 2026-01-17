use ecash_sdk_core::{ChainId, EasyCashClient, IntentType, SdkConfig, TransactionRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // 1. Initialize Client with custom config
    let cfg = SdkConfig::default_config()
        .with_api_key(std::env::var("ECASH_API_KEY").unwrap_or_default());

    let sdk = EasyCashClient::new(Some(cfg))?;

    println!("🚀 EasyCash SDK Initialized (Advanced Mode)");

    // 2. Define a Shielded Transfer Request
    let req = TransactionRequest {
        reference_id: "ref_pay_salary_001".to_string(),
        intent_type: IntentType::Transfer,
        amount: "5000.00".to_string(),
        asset: "USDC".to_string(),
        recipient: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string()),
        source_chain: ChainId::Base,
        target_chain: None,
        is_shielded: true, // Enable ZK Privacy
    };

    // 3. Execute
    println!(
        "Processing Transfer: {} {} (Shielded: {})...",
        req.amount, req.asset, req.is_shielded
    );

    let resp = sdk.execute_transaction(&req).await?;

    println!("✅ Success! Tx Hash: {}", resp.tx_hash);
    println!("   Block: {}", resp.block_height);
    println!("   Fee: {}", resp.fee_used);

    // 4. Display Metrics
    println!("\n📊 SDK Metrics:");
    let metrics = sdk.get_metrics();
    for (key, value) in metrics {
        println!("   {}: {}", key, value);
    }

    Ok(())
}
