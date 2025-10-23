use serde::{Deserialize, Serialize};

/// MEV risk score from AI engine (0.0 = safe, 1.0 = high risk)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MevRiskScore(pub f32);

impl MevRiskScore {
    pub fn new(score: f32) -> Self {
        Self(score.clamp(0.0, 1.0))
    }
    
    pub fn score(&self) -> f32 {
        self.0
    }

    pub fn is_high_risk(&self) -> bool {
        self.0 >= 0.8
    }

    pub fn is_medium_risk(&self) -> bool {
        self.0 >= 0.5 && self.0 < 0.8
    }

    pub fn is_low_risk(&self) -> bool {
        self.0 < 0.5
    }
}

/// Transaction status tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Submitted,
    Confirmed,
    Finalized,
    Failed(String),
    Expired,
}

/// Route type for multipath router
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteType {
    JitoBundle,
    JitoSingle,
    Firedancer,
    StandardRpc,
}

impl RouteType {
    pub fn requires_bundle(&self) -> bool {
        matches!(self, RouteType::JitoBundle)
    }
}
