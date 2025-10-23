pub mod features;
pub mod features_enhanced; // Production-ready 55-feature implementation
pub mod inference;
pub mod inference_enhanced; // Production-ready with drift detection
pub mod model;
pub mod pyth_oracle;
pub mod shadow_mode;
pub mod transaction_extractor;
pub mod validator_intel; // 241 malicious validators tracked

// NEW: Research-backed enhancements (October 2025)
pub mod drift_detection; // Multi-method ensemble (PSI + KS + JS)
pub mod enhanced_features; // 67 features with Jito bundle detection
pub mod adaptive_heuristics; // Dynamic thresholds + multi-stage filtering
pub mod firedancer_monitor; // Firedancer adoption tracking + new MEV patterns

pub use pyth_oracle::{PriceData, PythOracleClient};

// Export enhanced versions for production
pub use features_enhanced::{FeatureExtractor, FeatureVector, TransactionData, SwapDetailsData, ValidatorTracker};
pub use inference_enhanced::InferenceEngine;
pub use model::ModelConfig;
pub use shadow_mode::{ShadowConfig, ShadowModeManager, ShadowPrediction, ShadowStats};
pub use transaction_extractor::extract_from_transaction;
pub use validator_intel::{ValidatorIntel, load_validator_intel, calculate_validator_risk};

// Export new research-backed modules
pub use drift_detection::{DriftDetector, DriftScore, VotingStrategy};
pub use enhanced_features::{EnhancedFeatureVector, EnhancedTransactionData, JitoBundleInfo};
pub use adaptive_heuristics::{AdaptiveHeuristics, MEVDetectionPipeline, ThresholdConfig};
pub use firedancer_monitor::{
    FiredancerMonitor, FiredancerReport, FiredancerMevPattern, 
    FiredancerPerformance, AlertLevel, ValidatorClient
};
