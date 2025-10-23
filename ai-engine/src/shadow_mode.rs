//! Shadow Mode Implementation for MEV Detection
//!
//! Runs shadow model predictions alongside production without affecting users.
//! Based on industry best practices from Google, Stripe, and Riskified research.
//!
//! Key features:
//! - Feature flag control for instant rollback
//! - Async prediction logging (zero blocking)
//! - Correlation tracking (request_id)
//! - Buffered writes to disk
//! - Comprehensive metadata for analysis

use sentinel_core::{Result, SentinelError};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Shadow prediction result with metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShadowPrediction {
    /// Unique ID linking shadow to production prediction
    pub request_id: String,

    /// Timestamp when prediction was made (milliseconds since epoch)
    pub timestamp_ms: u64,

    /// Transaction signature
    pub signature: String,

    /// Shadow model version identifier
    pub model_version: String,

    /// MEV risk score from shadow model (0.0-1.0)
    pub shadow_risk_score: f32,

    /// Classification (true = MEV detected)
    pub shadow_is_mev: bool,

    /// Inference latency in microseconds
    pub latency_us: u64,

    /// Production model risk score (for comparison)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub production_risk_score: Option<f32>,

    /// Production model classification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub production_is_mev: Option<bool>,

    /// Transaction features (for drift analysis)
    pub features: serde_json::Value,

    /// Any errors during shadow prediction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Parameters for logging a shadow prediction
#[derive(Debug)]
pub struct ShadowLogParams {
    pub request_id: String,
    pub signature: String,
    pub shadow_risk_score: f32,
    pub shadow_is_mev: bool,
    pub latency_us: u64,
    pub production_risk_score: Option<f32>,
    pub production_is_mev: Option<bool>,
    pub features: serde_json::Value,
}

/// Configuration for shadow mode
#[derive(Debug, Clone)]
pub struct ShadowConfig {
    /// Maximum predictions to buffer before flush
    pub buffer_size: usize,

    /// Shadow model version identifier
    pub model_version: String,

    /// Log file path (JSONL format)
    pub log_path: String,

    /// Enable shadow mode on startup
    pub enabled_on_start: bool,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            model_version: "v1.0".to_string(),
            log_path: "logs/shadow_predictions.jsonl".to_string(),
            enabled_on_start: true,
        }
    }
}

/// Shadow mode manager
///
/// Manages shadow predictions, buffering, and logging.
/// Thread-safe and async-friendly.
pub struct ShadowModeManager {
    /// Feature flag: enable/disable shadow mode
    enabled: Arc<RwLock<bool>>,

    /// In-memory buffer for shadow predictions
    predictions: Arc<RwLock<Vec<ShadowPrediction>>>,

    /// Configuration
    config: ShadowConfig,
}

impl ShadowModeManager {
    /// Create new shadow mode manager
    pub fn new(config: ShadowConfig) -> Self {
        Self {
            enabled: Arc::new(RwLock::new(config.enabled_on_start)),
            predictions: Arc::new(RwLock::new(Vec::with_capacity(config.buffer_size))),
            config,
        }
    }

    /// Check if shadow mode is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Enable shadow mode
    pub async fn enable(&self) {
        *self.enabled.write().await = true;
        tracing::info!("🟢 Shadow mode enabled");
    }

    /// Disable shadow mode (instant rollback)
    pub async fn disable(&self) {
        *self.enabled.write().await = false;
        tracing::warn!("🔴 Shadow mode disabled - predictions will not be logged");
    }

    /// Log a shadow prediction (async, non-blocking)
    ///
    /// This is called from a background task and should never block
    /// the production prediction path.
    pub async fn log_prediction(&self, params: ShadowLogParams) -> Result<()> {
        let ShadowLogParams {
            request_id,
            signature,
            shadow_risk_score,
            shadow_is_mev,
            latency_us,
            production_risk_score,
            production_is_mev,
            features,
        } = params;
        if !self.is_enabled().await {
            return Ok(());
        }

        let prediction = ShadowPrediction {
            request_id,
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| SentinelError::InferenceError(format!("Time error: {}", e)))?
                .as_millis() as u64,
            signature,
            model_version: self.config.model_version.clone(),
            shadow_risk_score,
            shadow_is_mev,
            latency_us,
            production_risk_score,
            production_is_mev,
            features,
            error: None,
        };

        // Add to buffer
        let mut predictions = self.predictions.write().await;
        predictions.push(prediction);

        // Flush if buffer full
        if predictions.len() >= self.config.buffer_size {
            self.flush_internal(&mut predictions).await?;
        }

        Ok(())
    }

    /// Log a shadow prediction error
    pub async fn log_error(
        &self,
        request_id: String,
        signature: String,
        error: String,
    ) -> Result<()> {
        if !self.is_enabled().await {
            return Ok(());
        }

        let prediction = ShadowPrediction {
            request_id,
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| SentinelError::InferenceError(format!("Time error: {}", e)))?
                .as_millis() as u64,
            signature,
            model_version: self.config.model_version.clone(),
            shadow_risk_score: 0.0,
            shadow_is_mev: false,
            latency_us: 0,
            production_risk_score: None,
            production_is_mev: None,
            features: serde_json::json!({}),
            error: Some(error),
        };

        let mut predictions = self.predictions.write().await;
        predictions.push(prediction);

        if predictions.len() >= self.config.buffer_size {
            self.flush_internal(&mut predictions).await?;
        }

        Ok(())
    }

    /// Flush buffer to persistent storage
    pub async fn flush(&self) -> Result<()> {
        let mut predictions = self.predictions.write().await;
        self.flush_internal(&mut predictions).await
    }

    async fn flush_internal(&self, predictions: &mut Vec<ShadowPrediction>) -> Result<()> {
        if predictions.is_empty() {
            return Ok(());
        }

        tracing::info!(
            "📝 Flushing {} shadow predictions to {}",
            predictions.len(),
            self.config.log_path
        );

        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&self.config.log_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SentinelError::InferenceError(format!("Failed to create log dir: {}", e))
            })?;
        }

        // Write to JSONL file (append mode)
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_path)
            .map_err(|e| {
                SentinelError::InferenceError(format!("Failed to open log file: {}", e))
            })?;

        let mut writer = std::io::BufWriter::new(log_file);
        for pred in predictions.iter() {
            serde_json::to_writer(&mut writer, pred).map_err(|e| {
                SentinelError::InferenceError(format!("Failed to write JSON: {}", e))
            })?;
            writeln!(&mut writer).map_err(|e| {
                SentinelError::InferenceError(format!("Failed to write newline: {}", e))
            })?;
        }
        writer
            .flush()
            .map_err(|e| SentinelError::InferenceError(format!("Failed to flush buffer: {}", e)))?;

        tracing::info!("✅ Flushed {} predictions successfully", predictions.len());

        // Clear buffer
        predictions.clear();

        Ok(())
    }

    /// Get statistics for monitoring
    pub async fn get_stats(&self) -> ShadowStats {
        let predictions = self.predictions.read().await;
        let enabled = self.is_enabled().await;

        ShadowStats {
            enabled,
            buffered_predictions: predictions.len(),
            model_version: self.config.model_version.clone(),
            log_path: self.config.log_path.clone(),
        }
    }
}

/// Shadow mode statistics for monitoring
#[derive(Debug, Serialize, Clone)]
pub struct ShadowStats {
    pub enabled: bool,
    pub buffered_predictions: usize,
    pub model_version: String,
    pub log_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enable_disable() {
        let config = ShadowConfig::default();
        let manager = ShadowModeManager::new(config);

        assert!(manager.is_enabled().await);

        manager.disable().await;
        assert!(!manager.is_enabled().await);

        manager.enable().await;
        assert!(manager.is_enabled().await);
    }

    #[tokio::test]
    async fn test_log_prediction() {
        let config = ShadowConfig {
            buffer_size: 10,
            log_path: "logs/test_shadow.jsonl".to_string(),
            ..Default::default()
        };
        let manager = ShadowModeManager::new(config);

        let result = manager
            .log_prediction(ShadowLogParams {
                request_id: "test-123".to_string(),
                signature: "sig-456".to_string(),
                shadow_risk_score: 0.75,
                shadow_is_mev: true,
                latency_us: 1200,
                production_risk_score: Some(0.65),
                production_is_mev: Some(false),
                features: serde_json::json!({"fee": 1000}),
            })
            .await;

        assert!(result.is_ok());

        let stats = manager.get_stats().await;
        assert_eq!(stats.buffered_predictions, 1);
    }
}
