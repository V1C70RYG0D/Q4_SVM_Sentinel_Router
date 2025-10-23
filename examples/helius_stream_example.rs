//! Production Helius LaserStream Integration
//! 
//! Demonstrates production LaserStream client usage
//! for real-time Solana transaction ingestion and MEV detection.

use helius_ingestor::{LaserStreamGrpcClient, LaserStreamConfig, SubscriptionFilters, TransactionUpdate};
use tokio::sync::mpsc;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Get Helius API key from environment
    let api_key = env::var("HELIUS_API_KEY")
        .expect("HELIUS_API_KEY must be set");

    // Configure LaserStream connection
    let config = LaserStreamConfig {
        api_key: api_key.clone(),
        endpoint: "https://mainnet.helius-rpc.com".to_string(),
        enable_replay: true,
        max_reconnect_attempts: 10,
        reconnect_delay_ms: 5000,
        auth_token: Some(api_key),
    };

    // Configure filters for DEX transactions only
    let filters = SubscriptionFilters::dex_programs();

    println!("🚀 Starting Helius LaserStream client...");
    println!("📡 Endpoint: {}", config.endpoint);
    println!("🔍 Filters: DEX transactions (Raydium, Orca, Jupiter)");
    println!("🔄 Replay enabled: {}", config.enable_replay);
    println!();

    // Create client
    let mut client = LaserStreamGrpcClient::new(config, filters);

    // Create channel for receiving updates
    let (tx, mut rx) = mpsc::unbounded_channel::<TransactionUpdate>();

    // Start streaming in background
    let stream_handle = tokio::spawn(async move {
        if let Err(e) = client.start_stream(tx).await {
            eprintln!("❌ Stream error: {:?}", e);
        }
    });

    // Process transactions
    let mut tx_count = 0u64;
    let mut mev_candidates = 0u64;

    while let Some(update) = rx.recv().await {
        tx_count += 1;

        // Simple MEV detection heuristic
        let is_mev_candidate = update.success 
            && update.compute_units_consumed.unwrap_or(0) > 200_000
            && update.fee > 10_000;

        if is_mev_candidate {
            mev_candidates += 1;
            println!(
                "🎯 MEV Candidate #{}: sig={} slot={} fee={} cu={}",
                mev_candidates,
                &update.signature[..8],
                update.slot,
                update.fee,
                update.compute_units_consumed.unwrap_or(0)
            );
        }

        // Print status every 100 transactions
        if tx_count % 100 == 0 {
            let mev_rate = (mev_candidates as f64 / tx_count as f64) * 100.0;
            println!(
                "📊 Processed: {} txs | MEV candidates: {} ({:.2}%)",
                tx_count, mev_candidates, mev_rate
            );
        }

        // Run for 1000 transactions then exit (remove limit for production)
        if tx_count >= 1000 {
            println!("\n✅ Processed 1000 transactions, shutting down...");
            break;
        }
    }

    // Cleanup
    stream_handle.abort();

    println!("\n📈 Final Statistics:");
    println!("   Total transactions: {}", tx_count);
    println!("   MEV candidates: {}", mev_candidates);
    println!("   Detection rate: {:.2}%", (mev_candidates as f64 / tx_count as f64) * 100.0);

    Ok(())
}
