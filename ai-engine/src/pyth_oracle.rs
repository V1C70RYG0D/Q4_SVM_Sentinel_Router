use reqwest::Client;
use sentinel_core::{Result, SentinelError};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Pyth oracle client for real-time price feeds via HTTP API
pub struct PythOracleClient {
    http_client: Client,
    api_endpoint: String,
    price_feed_ids: HashMap<String, String>,
    cache: HashMap<String, CachedPrice>,
    cache_ttl: Duration,
}

impl PythOracleClient {
    pub fn new(api_endpoint: String, cache_ttl_secs: u64) -> Self {
        let http_client = Client::new();

        // Pyth price feed IDs (use HTTP API instead of on-chain)
        let mut price_feed_ids = HashMap::new();

        // SOL/USD feed ID
        price_feed_ids.insert(
            "SOL/USD".to_string(),
            "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d".to_string(),
        );

        // USDC/USD feed ID
        price_feed_ids.insert(
            "USDC/USD".to_string(),
            "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a".to_string(),
        );

        Self {
            http_client,
            api_endpoint,
            price_feed_ids,
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
        }
    }

    /// Create client for Pyth Hermes API
    pub fn hermes_devnet() -> Self {
        Self::new(
            "https://hermes.pyth.network".to_string(),
            1, // 1 second cache TTL
        )
    }

    /// Get price for a symbol pair (e.g., "SOL/USD")
    pub async fn get_price(&mut self, symbol: &str) -> Result<PriceData> {
        // Check cache first
        if let Some(cached) = self.cache.get(symbol) {
            if cached.timestamp.elapsed() < self.cache_ttl {
                debug!("Cache hit for {}: ${}", symbol, cached.price.price);
                return Ok(cached.price.clone());
            }
        }

        // Fetch from Pyth HTTP API
        let feed_id = self.price_feed_ids.get(symbol).ok_or_else(|| {
            SentinelError::PriceOracleError(format!("Unknown symbol: {}", symbol))
        })?;

        let url = format!(
            "{}/v2/updates/price/latest?ids[]=0x{}",
            self.api_endpoint, feed_id
        );

        let response = self.http_client.get(&url).send().await.map_err(|e| {
            SentinelError::PriceOracleError(format!("Failed to fetch price: {}", e))
        })?;

        let price_response: PythPriceResponse = response.json().await.map_err(|e| {
            SentinelError::PriceOracleError(format!("Failed to parse price response: {}", e))
        })?;

        let parsed_price = price_response.parsed.first().ok_or_else(|| {
            SentinelError::PriceOracleError("No price data in response".to_string())
        })?;

        let price_update = &parsed_price.price;

        let price_data = PriceData {
            symbol: symbol.to_string(),
            price: price_update.price.parse::<f64>().unwrap_or(0.0)
                * 10_f64.powi(price_update.expo),
            conf: price_update.conf.parse::<f64>().unwrap_or(0.0) * 10_f64.powi(price_update.expo),
            expo: price_update.expo,
            publish_time: price_update.publish_time,
        };

        // Update cache
        self.cache.insert(
            symbol.to_string(),
            CachedPrice {
                price: price_data.clone(),
                timestamp: Instant::now(),
            },
        );

        info!(
            "Fetched {} price: ${} (±${})",
            symbol, price_data.price, price_data.conf
        );

        Ok(price_data)
    }

    /// Calculate price impact for a swap
    pub async fn calculate_price_impact(
        &mut self,
        input_symbol: &str,
        output_symbol: &str,
        input_amount: u64,
        expected_output: u64,
    ) -> Result<f64> {
        let input_price = self.get_price(input_symbol).await?;
        let output_price = self.get_price(output_symbol).await?;

        let input_value_usd = (input_amount as f64) * input_price.price;
        let output_value_usd = (expected_output as f64) * output_price.price;

        let impact = ((output_value_usd - input_value_usd) / input_value_usd).abs();

        debug!(
            "Price impact: {:.2}% (input: ${}, output: ${})",
            impact * 100.0,
            input_value_usd,
            output_value_usd
        );

        Ok(impact)
    }

    /// Batch get multiple prices
    pub async fn get_prices(&mut self, symbols: &[&str]) -> Result<HashMap<String, PriceData>> {
        let mut prices = HashMap::new();

        for symbol in symbols {
            match self.get_price(symbol).await {
                Ok(price) => {
                    prices.insert(symbol.to_string(), price);
                }
                Err(e) => {
                    warn!("Failed to get price for {}: {:?}", symbol, e);
                }
            }
        }

        Ok(prices)
    }
}

#[derive(Debug, Clone)]
pub struct PriceData {
    pub symbol: String,
    pub price: f64,
    pub conf: f64, // Confidence interval
    pub expo: i32,
    pub publish_time: i64,
}

struct CachedPrice {
    price: PriceData,
    timestamp: Instant,
}

// Pyth HTTP API response types
#[derive(Debug, Deserialize)]
struct PythPriceResponse {
    parsed: Vec<ParsedPrice>,
}

#[derive(Debug, Deserialize)]
struct ParsedPrice {
    #[allow(dead_code)] // Required for deserialization
    id: String,
    price: PriceInfo,
}

#[derive(Debug, Deserialize)]
struct PriceInfo {
    price: String,
    conf: String,
    expo: i32,
    publish_time: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = PythOracleClient::hermes_devnet();
        assert!(client.price_feed_ids.contains_key("SOL/USD"));
        assert_eq!(client.api_endpoint, "https://hermes.pyth.network");
    }
}
