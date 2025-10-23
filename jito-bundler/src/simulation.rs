use sentinel_core::Result;
use tracing::{info, warn};

use crate::builder::JitoBundle;
use crate::jito_client::JitoClient;

/// Production-ready bundle simulator using JitoClient
pub struct BundleSimulator {
    client: JitoClient,
}

impl BundleSimulator {
    /// Create new simulator for devnet
    pub fn devnet() -> sentinel_core::Result<Self> {
        Ok(Self {
            client: JitoClient::devnet()?,
        })
    }

    /// Create new simulator for mainnet
    pub fn mainnet() -> sentinel_core::Result<Self> {
        Ok(Self {
            client: JitoClient::mainnet()?,
        })
    }

    /// Create new simulator with custom endpoint
    pub fn new(block_engine_url: String) -> sentinel_core::Result<Self> {
        Ok(Self {
            client: JitoClient::new(block_engine_url)?,
        })
    }

    /// Simulate bundle execution before submission
    /// This uses Jito's simulateBundle RPC method
    pub async fn simulate(&self, bundle: &JitoBundle) -> Result<SimulationResult> {
        info!(
            "Simulating bundle with {} transactions",
            bundle.transactions.len()
        );

        // Call real Jito simulateBundle RPC
        let jito_result = self.client.simulate_bundle(&bundle.transactions).await?;

        // Convert Jito result to our simplified result
        let success =
            jito_result.results.is_empty() || jito_result.results.iter().all(|r| r.err.is_none());

        let error = jito_result.results.iter().find_map(|r| r.err.clone());

        let logs: Vec<String> = jito_result
            .results
            .iter()
            .flat_map(|r| r.logs.clone())
            .collect();

        let compute_units_consumed = jito_result
            .results
            .iter()
            .filter_map(|r| r.units_consumed)
            .sum();

        let result = SimulationResult {
            success,
            error,
            logs,
            compute_units_consumed,
        };

        if result.success {
            info!(
                "Bundle simulation successful - {} CUs consumed",
                result.compute_units_consumed
            );
        } else {
            warn!("Bundle simulation failed: {:?}", result.error);
        }

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub success: bool,
    pub error: Option<String>,
    pub logs: Vec<String>,
    pub compute_units_consumed: u64,
}

impl SimulationResult {
    pub fn is_success(&self) -> bool {
        self.success && self.error.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simulation() {
        let _simulator = BundleSimulator::new("http://localhost:8899".to_string());
        let _bundle = JitoBundle::new();
        // Would need valid transactions for full test
        // This test ensures the types compile correctly
    }
}
