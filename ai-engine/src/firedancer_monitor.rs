/// Firedancer Validator Client Monitoring
/// 
/// Monitors Solana validator infrastructure for Firedancer adoption and new MEV patterns.
/// 
/// Research context (Oct 2025):
/// - Firedancer is Jump Crypto's high-performance validator client
/// - Expected to introduce new MEV extraction patterns
/// - Alpenglow upgrade (150ms finality) creates new MEV opportunities
/// - 99% validator consensus on faster finality = MEV landscape shift
/// 
/// This module provides:
/// 1. Firedancer adoption tracking across validator network
/// 2. New MEV pattern detection (Firedancer-specific)
/// 3. Performance comparison (Jito vs Firedancer)
/// 4. Alert system for drift in MEV strategies
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Firedancer adoption and MEV monitoring dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiredancerMonitor {
    /// Current Firedancer adoption rate (% of stake)
    pub adoption_rate_pct: f32,
    
    /// Validators running Firedancer (pubkey -> version)
    pub firedancer_validators: HashMap<String, String>,
    
    /// Detected MEV patterns unique to Firedancer
    pub firedancer_mev_patterns: Vec<FiredancerMevPattern>,
    
    /// Performance comparison metrics
    pub performance_metrics: FiredancerPerformance,
    
    /// Last updated timestamp
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiredancerMevPattern {
    /// Pattern identifier
    pub pattern_id: String,
    
    /// Pattern description
    pub description: String,
    
    /// Detection count in last 24h
    pub detection_count_24h: u32,
    
    /// Average MEV extracted per occurrence (SOL)
    pub avg_mev_extracted_sol: f64,
    
    /// Confidence score (0-1)
    pub confidence: f32,
    
    /// First detected timestamp
    pub first_detected: DateTime<Utc>,
    
    /// Example transaction signatures
    pub example_signatures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiredancerPerformance {
    /// Average block production time (ms)
    pub avg_block_time_ms: f32,
    
    /// Transaction throughput (TPS)
    pub avg_tps: u32,
    
    /// MEV opportunities captured vs Jito
    pub mev_capture_rate_vs_jito: f32,
    
    /// Skip rate (missed slots %)
    pub skip_rate_pct: f32,
    
    /// Comparison period
    pub measurement_period_hours: u32,
}

impl Default for FiredancerMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl FiredancerMonitor {
    /// Initialize Firedancer monitoring
    pub fn new() -> Self {
        Self {
            adoption_rate_pct: 0.0,
            firedancer_validators: HashMap::new(),
            firedancer_mev_patterns: Vec::new(),
            performance_metrics: FiredancerPerformance::default(),
            last_update: Utc::now(),
        }
    }
    
    /// Update adoption metrics from validator registry
    pub fn update_adoption(&mut self, validators: HashMap<String, ValidatorInfo>) {
        let mut total_stake: u64 = 0;
        let mut firedancer_stake: u64 = 0;
        
        self.firedancer_validators.clear();
        
        for (pubkey, info) in validators {
            total_stake += info.stake;
            
            if info.client_type == ValidatorClient::Firedancer {
                firedancer_stake += info.stake;
                self.firedancer_validators.insert(pubkey, info.version);
            }
        }
        
        self.adoption_rate_pct = if total_stake > 0 {
            (firedancer_stake as f32 / total_stake as f32) * 100.0
        } else {
            0.0
        };
        
        self.last_update = Utc::now();
        
        // Log significant adoption milestones
        if self.adoption_rate_pct >= 10.0 && self.adoption_rate_pct < 11.0 {
            tracing::info!("🚀 Firedancer adoption reached 10% of network stake");
        } else if self.adoption_rate_pct >= 25.0 && self.adoption_rate_pct < 26.0 {
            tracing::warn!("⚠️  Firedancer adoption reached 25% - Monitor for new MEV patterns");
        } else if self.adoption_rate_pct >= 50.0 {
            tracing::warn!("🔴 ALERT: Firedancer adoption >50% - Major MEV landscape shift expected");
        }
    }
    
    /// Detect Firedancer-specific MEV patterns
    /// 
    /// Patterns to monitor:
    /// 1. Ultra-fast finality exploitation (150ms window)
    /// 2. New bundle formats unique to Firedancer
    /// 3. Cross-client arbitrage (Jito vs Firedancer timing)
    pub fn detect_firedancer_patterns(&mut self, transactions: &[TransactionContext]) -> Vec<FiredancerMevPattern> {
        let mut detected_patterns = Vec::new();
        
        // Pattern 1: Ultra-fast finality exploitation
        let ultra_fast_finality = self.detect_ultra_fast_finality(transactions);
        if let Some(pattern) = ultra_fast_finality {
            detected_patterns.push(pattern);
        }
        
        // Pattern 2: Cross-client timing arbitrage
        let cross_client_arb = self.detect_cross_client_arbitrage(transactions);
        if let Some(pattern) = cross_client_arb {
            detected_patterns.push(pattern);
        }
        
        // Pattern 3: Firedancer bundle format detection
        let new_bundle_format = self.detect_new_bundle_format(transactions);
        if let Some(pattern) = new_bundle_format {
            detected_patterns.push(pattern);
        }
        
        // Update internal pattern registry
        for pattern in &detected_patterns {
            self.firedancer_mev_patterns.push(pattern.clone());
        }
        
        // Keep only last 24h of patterns
        self.prune_old_patterns();
        
        detected_patterns
    }
    
    /// Detect ultra-fast finality exploitation (Alpenglow 150ms)
    fn detect_ultra_fast_finality(&self, transactions: &[TransactionContext]) -> Option<FiredancerMevPattern> {
        // Look for MEV strategies exploiting <200ms finality window
        let mut ultra_fast_count = 0;
        let mut example_sigs = Vec::new();
        
        for tx in transactions {
            // Check if transaction leverages sub-200ms finality
            if tx.finality_time_ms < 200 && tx.is_mev_suspect {
                ultra_fast_count += 1;
                if example_sigs.len() < 3 {
                    example_sigs.push(tx.signature.clone());
                }
            }
        }
        
        if ultra_fast_count > 10 {
            // Estimate MEV based on detection frequency and typical arbitrage profits
            let estimated_mev = (ultra_fast_count as f64 * 0.045).min(2.5);
            Some(FiredancerMevPattern {
                pattern_id: "FIREDANCER_ULTRA_FAST_FINALITY".to_string(),
                description: "MEV extraction leveraging <150ms finality (Alpenglow upgrade)".to_string(),
                detection_count_24h: ultra_fast_count,
                avg_mev_extracted_sol: estimated_mev,
                confidence: 0.85,
                first_detected: Utc::now(),
                example_signatures: example_sigs,
            })
        } else {
            None
        }
    }
    
    /// Detect cross-client timing arbitrage (Jito vs Firedancer)
    fn detect_cross_client_arbitrage(&self, transactions: &[TransactionContext]) -> Option<FiredancerMevPattern> {
        // Look for patterns where bots exploit timing differences between clients
        let mut cross_client_count = 0;
        let mut example_sigs = Vec::new();
        
        for tx in transactions {
            if tx.exploits_client_timing {
                cross_client_count += 1;
                if example_sigs.len() < 3 {
                    example_sigs.push(tx.signature.clone());
                }
            }
        }
        
        if cross_client_count > 5 {
            // Estimate MEV for cross-client timing exploitation
            let estimated_mev = (cross_client_count as f64 * 0.15).min(5.0);
            Some(FiredancerMevPattern {
                pattern_id: "CROSS_CLIENT_TIMING_ARB".to_string(),
                description: "Arbitrage exploiting Jito vs Firedancer block production timing".to_string(),
                detection_count_24h: cross_client_count,
                avg_mev_extracted_sol: estimated_mev,
                confidence: 0.75,
                first_detected: Utc::now(),
                example_signatures: example_sigs,
            })
        } else {
            None
        }
    }
    
    /// Detect new bundle format unique to Firedancer
    fn detect_new_bundle_format(&self, transactions: &[TransactionContext]) -> Option<FiredancerMevPattern> {
        // Firedancer may introduce new transaction bundling mechanisms
        let mut new_format_count = 0;
        let mut example_sigs = Vec::new();
        
        for tx in transactions {
            if tx.uses_non_jito_bundle {
                new_format_count += 1;
                if example_sigs.len() < 3 {
                    example_sigs.push(tx.signature.clone());
                }
            }
        }
        
        if new_format_count > 8 {
            // Estimate MEV for new bundling mechanisms
            let estimated_mev = (new_format_count as f64 * 0.08).min(3.0);
            Some(FiredancerMevPattern {
                pattern_id: "FIREDANCER_NEW_BUNDLE_FORMAT".to_string(),
                description: "Non-Jito bundle format detected (potential Firedancer native bundling)".to_string(),
                detection_count_24h: new_format_count,
                avg_mev_extracted_sol: estimated_mev,
                confidence: 0.70,
                first_detected: Utc::now(),
                example_signatures: example_sigs,
            })
        } else {
            None
        }
    }
    
    /// Prune patterns older than 24h
    fn prune_old_patterns(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        self.firedancer_mev_patterns.retain(|p| p.first_detected > cutoff);
    }
    
    /// Generate monitoring report
    pub fn generate_report(&self) -> FiredancerReport {
        FiredancerReport {
            adoption_rate_pct: self.adoption_rate_pct,
            total_firedancer_validators: self.firedancer_validators.len(),
            active_patterns: self.firedancer_mev_patterns.len(),
            performance: self.performance_metrics.clone(),
            alert_level: self.calculate_alert_level(),
            recommendations: self.generate_recommendations(),
            report_timestamp: Utc::now(),
        }
    }
    
    /// Calculate alert level based on adoption and pattern detection
    fn calculate_alert_level(&self) -> AlertLevel {
        if self.adoption_rate_pct >= 50.0 {
            AlertLevel::Critical
        } else if self.adoption_rate_pct >= 25.0 || self.firedancer_mev_patterns.len() > 5 {
            AlertLevel::Warning
        } else if self.adoption_rate_pct >= 10.0 {
            AlertLevel::Info
        } else {
            AlertLevel::Normal
        }
    }
    
    /// Generate actionable recommendations
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.adoption_rate_pct >= 25.0 {
            recommendations.push(
                "HIGH ADOPTION: Update feature engineering to include Firedancer-specific signals".to_string()
            );
        }
        
        if self.firedancer_mev_patterns.len() > 3 {
            recommendations.push(
                format!("NEW PATTERNS: {} Firedancer-specific MEV patterns detected - Consider model retraining", 
                    self.firedancer_mev_patterns.len())
            );
        }
        
        if self.performance_metrics.mev_capture_rate_vs_jito > 1.2 {
            recommendations.push(
                "PERFORMANCE: Firedancer validators capturing 20%+ more MEV than Jito - Monitor for network effects".to_string()
            );
        }
        
        if recommendations.is_empty() {
            recommendations.push("No immediate action required - Continue monitoring".to_string());
        }
        
        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiredancerReport {
    pub adoption_rate_pct: f32,
    pub total_firedancer_validators: usize,
    pub active_patterns: usize,
    pub performance: FiredancerPerformance,
    pub alert_level: AlertLevel,
    pub recommendations: Vec<String>,
    pub report_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertLevel {
    Normal,
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub stake: u64,
    pub client_type: ValidatorClient,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidatorClient {
    Jito,
    Firedancer,
    Anza,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TransactionContext {
    pub signature: String,
    pub finality_time_ms: u64,
    pub is_mev_suspect: bool,
    pub exploits_client_timing: bool,
    pub uses_non_jito_bundle: bool,
}

impl Default for FiredancerPerformance {
    fn default() -> Self {
        Self {
            avg_block_time_ms: 400.0,
            avg_tps: 3000,
            mev_capture_rate_vs_jito: 1.0,
            skip_rate_pct: 0.5,
            measurement_period_hours: 24,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_firedancer_monitor_initialization() {
        let monitor = FiredancerMonitor::new();
        assert_eq!(monitor.adoption_rate_pct, 0.0);
        assert_eq!(monitor.firedancer_validators.len(), 0);
    }
    
    #[test]
    fn test_adoption_rate_calculation() {
        let mut monitor = FiredancerMonitor::new();
        
        let mut validators = HashMap::new();
        validators.insert("val1".to_string(), ValidatorInfo {
            stake: 1_000_000,
            client_type: ValidatorClient::Firedancer,
            version: "1.0.0".to_string(),
        });
        validators.insert("val2".to_string(), ValidatorInfo {
            stake: 3_000_000,
            client_type: ValidatorClient::Jito,
            version: "1.18.0".to_string(),
        });
        
        monitor.update_adoption(validators);
        
        // 1M / 4M = 25%
        assert!((monitor.adoption_rate_pct - 25.0).abs() < 0.1);
        assert_eq!(monitor.firedancer_validators.len(), 1);
    }
    
    #[test]
    fn test_alert_level_calculation() {
        let mut monitor = FiredancerMonitor::new();
        
        // Test normal level
        monitor.adoption_rate_pct = 5.0;
        assert_eq!(monitor.calculate_alert_level(), AlertLevel::Normal);
        
        // Test info level
        monitor.adoption_rate_pct = 15.0;
        assert_eq!(monitor.calculate_alert_level(), AlertLevel::Info);
        
        // Test warning level
        monitor.adoption_rate_pct = 30.0;
        assert_eq!(monitor.calculate_alert_level(), AlertLevel::Warning);
        
        // Test critical level
        monitor.adoption_rate_pct = 55.0;
        assert_eq!(monitor.calculate_alert_level(), AlertLevel::Critical);
    }
    
    #[test]
    fn test_pattern_detection() {
        let monitor = FiredancerMonitor::new();
        
        let transactions = vec![
            TransactionContext {
                signature: "sig1".to_string(),
                finality_time_ms: 150,
                is_mev_suspect: true,
                exploits_client_timing: false,
                uses_non_jito_bundle: false,
            },
            TransactionContext {
                signature: "sig2".to_string(),
                finality_time_ms: 180,
                is_mev_suspect: true,
                exploits_client_timing: false,
                uses_non_jito_bundle: false,
            },
        ];
        
        let pattern = monitor.detect_ultra_fast_finality(&transactions);
        assert!(pattern.is_none()); // Need >10 detections
    }
}
