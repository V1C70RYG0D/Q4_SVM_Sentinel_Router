use reqwest::Client;
use sentinel_core::{Result, SentinelError};
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::Transaction;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Production Jito Block Engine client
pub struct JitoClient {
    http_client: Client,
    block_engine_url: String,
}

impl JitoClient {
    /// Create new Jito client for devnet or mainnet
    pub fn new(block_engine_url: String) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| SentinelError::NetworkError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            http_client,
            block_engine_url,
        })
    }

    /// Create devnet client
    pub fn devnet() -> Result<Self> {
        Self::new("https://frankfurt.devnet.block-engine.jito.wtf".to_string())
    }

    /// Create mainnet client  
    pub fn mainnet() -> Result<Self> {
        Self::new("https://mainnet.block-engine.jito.wtf".to_string())
    }

    /// Get the block engine URL
    pub fn block_engine_url(&self) -> &str {
        &self.block_engine_url
    }

    /// Simulate a bundle before sending
    pub async fn simulate_bundle(&self, transactions: &[Transaction]) -> Result<SimulationResult> {
        let serialized_txs: Vec<String> = transactions
            .iter()
            .map(|tx| {
                use base64::engine::general_purpose::STANDARD as BASE64;
                use base64::Engine;
                let bytes = bincode::serialize(tx)
                    .map_err(|e| SentinelError::SerializationError(e.to_string()))?;
                Ok(BASE64.encode(&bytes))
            })
            .collect::<Result<Vec<_>>>()?;

        let request = SimulateBundleRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "simulateBundle".to_string(),
            params: vec![serialized_txs],
        };

        info!("Simulating bundle with {} transactions", transactions.len());

        let response = self
            .http_client
            .post(format!("{}/api/v1/bundles", self.block_engine_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| SentinelError::RpcError(format!("Simulation request failed: {}", e)))?;

        let result: SimulateBundleResponse = response
            .json()
            .await
            .map_err(|e| SentinelError::RpcError(format!("Failed to parse simulation: {}", e)))?;

        if let Some(error) = result.error {
            return Err(SentinelError::BundleError(format!(
                "Simulation failed: {}",
                error.message
            )));
        }

        Ok(result.result.unwrap_or_default())
    }

    /// Send a bundle to Jito Block Engine
    pub async fn send_bundle(&self, transactions: &[Transaction]) -> Result<String> {
        let serialized_txs: Vec<String> = transactions
            .iter()
            .map(|tx| {
                use base64::engine::general_purpose::STANDARD as BASE64;
                use base64::Engine;
                let bytes = bincode::serialize(tx)
                    .map_err(|e| SentinelError::SerializationError(e.to_string()))?;
                Ok(BASE64.encode(&bytes))
            })
            .collect::<Result<Vec<_>>>()?;

        let request = SendBundleRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "sendBundle".to_string(),
            params: vec![serialized_txs],
        };

        info!(
            "Sending bundle with {} transactions to Jito",
            transactions.len()
        );

        let response = self
            .http_client
            .post(format!("{}/api/v1/bundles", self.block_engine_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| SentinelError::RpcError(format!("Send bundle failed: {}", e)))?;

        let result: SendBundleResponse = response
            .json()
            .await
            .map_err(|e| SentinelError::RpcError(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = result.error {
            return Err(SentinelError::BundleError(format!(
                "Send bundle failed: {}",
                error.message
            )));
        }

        let bundle_id = result
            .result
            .ok_or_else(|| SentinelError::BundleError("No bundle ID returned".to_string()))?;

        info!("Bundle sent successfully: {}", bundle_id);
        Ok(bundle_id)
    }

    /// Get inflight bundle statuses (for bundles within 5 minutes)
    /// This method provides near real-time feedback on recently submitted bundles
    pub async fn get_inflight_bundle_statuses(
        &self,
        bundle_ids: &[String],
    ) -> Result<Vec<BundleStatus>> {
        let request = GetInflightBundleStatusesRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getInflightBundleStatuses".to_string(),
            params: vec![bundle_ids.to_vec()],
        };

        debug!("Checking inflight status for {} bundles", bundle_ids.len());

        let response = self
            .http_client
            .post(format!("{}/api/v1/bundles", self.block_engine_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| SentinelError::RpcError(format!("Inflight status check failed: {}", e)))?;

        let result: GetInflightBundleStatusesResponse = response.json().await.map_err(|e| {
            SentinelError::RpcError(format!("Failed to parse inflight status: {}", e))
        })?;

        if let Some(error) = result.error {
            return Err(SentinelError::BundleError(format!(
                "Inflight status check failed: {}",
                error.message
            )));
        }

        Ok(result.result.unwrap_or_default().value)
    }

    /// Get bundle status
    pub async fn get_bundle_statuses(&self, bundle_ids: &[String]) -> Result<Vec<BundleStatus>> {
        let request = GetBundleStatusesRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getBundleStatuses".to_string(),
            params: vec![bundle_ids.to_vec()],
        };

        debug!("Checking status for {} bundles", bundle_ids.len());

        let response = self
            .http_client
            .post(format!("{}/api/v1/bundles", self.block_engine_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| SentinelError::RpcError(format!("Status check failed: {}", e)))?;

        let result: GetBundleStatusesResponse = response
            .json()
            .await
            .map_err(|e| SentinelError::RpcError(format!("Failed to parse status: {}", e)))?;

        if let Some(error) = result.error {
            return Err(SentinelError::BundleError(format!(
                "Status check failed: {}",
                error.message
            )));
        }

        Ok(result.result.unwrap_or_default().value)
    }

    /// Wait for bundle to land or fail
    pub async fn wait_for_bundle(
        &self,
        bundle_id: &str,
        timeout: Duration,
    ) -> Result<BundleStatus> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                warn!("Bundle {} timed out after {:?}", bundle_id, timeout);
                return Ok(BundleStatus {
                    bundle_id: bundle_id.to_string(),
                    status: "Timeout".to_string(),
                    landed_slot: None,
                });
            }

            let statuses = self.get_bundle_statuses(&[bundle_id.to_string()]).await?;

            if let Some(status) = statuses.first() {
                match status.status.as_str() {
                    "Landed" => {
                        info!(
                            "Bundle {} landed at slot {:?}",
                            bundle_id, status.landed_slot
                        );
                        return Ok(status.clone());
                    }
                    "Failed" | "Invalid" => {
                        warn!("Bundle {} failed: {}", bundle_id, status.status);
                        return Ok(status.clone());
                    }
                    "Pending" | "Processing" => {
                        debug!("Bundle {} still pending", bundle_id);
                    }
                    _ => {
                        debug!("Bundle {} status: {}", bundle_id, status.status);
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}

// Request/Response types
#[derive(Serialize)]
struct SimulateBundleRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct SimulateBundleResponse {
    result: Option<SimulationResult>,
    error: Option<RpcError>,
}

#[derive(Deserialize, Default)]
pub struct SimulationResult {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub results: Vec<TransactionResult>,
}

#[derive(Deserialize)]
pub struct TransactionResult {
    pub err: Option<String>,
    #[serde(default)]
    pub logs: Vec<String>,
    #[serde(default)]
    pub units_consumed: Option<u64>,
}

#[derive(Serialize)]
struct SendBundleRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct SendBundleResponse {
    result: Option<String>,
    error: Option<RpcError>,
}

#[derive(Serialize)]
struct GetInflightBundleStatusesRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct GetInflightBundleStatusesResponse {
    result: Option<BundleStatusesResult>,
    error: Option<RpcError>,
}

#[derive(Serialize)]
struct GetBundleStatusesRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct GetBundleStatusesResponse {
    result: Option<BundleStatusesResult>,
    error: Option<RpcError>,
}

#[derive(Deserialize, Default)]
struct BundleStatusesResult {
    value: Vec<BundleStatus>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BundleStatus {
    pub bundle_id: String,
    pub status: String, // "Pending", "Landed", "Failed", "Invalid"
    pub landed_slot: Option<u64>,
}

#[derive(Deserialize)]
struct RpcError {
    #[allow(dead_code)]
    code: i64,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = JitoClient::devnet().unwrap();
        assert!(client.block_engine_url().contains("devnet"));
    }

    #[test]
    fn test_mainnet_client() {
        let client = JitoClient::mainnet().unwrap();
        assert!(client.block_engine_url().contains("mainnet"));
    }
}
