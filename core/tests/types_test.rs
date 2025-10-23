//! Sentinel Core Types Tests
//! Tests MEV risk scores, transaction status, and route types

use sentinel_core::{MevRiskScore, RouteType, TransactionStatus};

/// Test: Create MEV risk score
#[test]
fn test_create_mev_risk_score() {
    let score = MevRiskScore::new(0.5);
    
    assert_eq!(score.score(), 0.5);
}

/// Test: MEV risk score clamping (max)
#[test]
fn test_mev_risk_score_clamp_max() {
    let score = MevRiskScore::new(1.5);
    
    assert_eq!(score.score(), 1.0, "Score should be clamped to 1.0");
}

/// Test: MEV risk score clamping (min)
#[test]
fn test_mev_risk_score_clamp_min() {
    let score = MevRiskScore::new(-0.5);
    
    assert_eq!(score.score(), 0.0, "Score should be clamped to 0.0");
}

/// Test: High risk detection
#[test]
fn test_high_risk_detection() {
    let high_risk = MevRiskScore::new(0.9);
    let not_high_risk = MevRiskScore::new(0.7);
    
    assert!(high_risk.is_high_risk());
    assert!(!not_high_risk.is_high_risk());
}

/// Test: Medium risk detection
#[test]
fn test_medium_risk_detection() {
    let medium_risk = MevRiskScore::new(0.6);
    let low_risk = MevRiskScore::new(0.4);
    let high_risk = MevRiskScore::new(0.9);
    
    assert!(medium_risk.is_medium_risk());
    assert!(!low_risk.is_medium_risk());
    assert!(!high_risk.is_medium_risk());
}

/// Test: Low risk detection
#[test]
fn test_low_risk_detection() {
    let low_risk = MevRiskScore::new(0.3);
    let medium_risk = MevRiskScore::new(0.6);
    
    assert!(low_risk.is_low_risk());
    assert!(!medium_risk.is_low_risk());
}

/// Test: Risk score boundary (0.8 threshold)
#[test]
fn test_risk_score_high_boundary() {
    let exactly_high = MevRiskScore::new(0.8);
    let just_below = MevRiskScore::new(0.799);
    
    assert!(exactly_high.is_high_risk());
    assert!(!just_below.is_high_risk());
    assert!(just_below.is_medium_risk());
}

/// Test: Risk score boundary (0.5 threshold)
#[test]
fn test_risk_score_medium_boundary() {
    let exactly_medium = MevRiskScore::new(0.5);
    let just_below = MevRiskScore::new(0.499);
    
    assert!(exactly_medium.is_medium_risk());
    assert!(!just_below.is_medium_risk());
    assert!(just_below.is_low_risk());
}

/// Test: Zero risk score
#[test]
fn test_zero_risk_score() {
    let zero_risk = MevRiskScore::new(0.0);
    
    assert_eq!(zero_risk.score(), 0.0);
    assert!(zero_risk.is_low_risk());
    assert!(!zero_risk.is_medium_risk());
    assert!(!zero_risk.is_high_risk());
}

/// Test: Maximum risk score
#[test]
fn test_max_risk_score() {
    let max_risk = MevRiskScore::new(1.0);
    
    assert_eq!(max_risk.score(), 1.0);
    assert!(max_risk.is_high_risk());
    assert!(!max_risk.is_medium_risk());
    assert!(!max_risk.is_low_risk());
}

/// Test: Transaction status variants
#[test]
fn test_transaction_status_variants() {
    let pending = TransactionStatus::Pending;
    let submitted = TransactionStatus::Submitted;
    let confirmed = TransactionStatus::Confirmed;
    let finalized = TransactionStatus::Finalized;
    let failed = TransactionStatus::Failed("InsufficientFunds".to_string());
    let expired = TransactionStatus::Expired;
    
    assert_eq!(pending, TransactionStatus::Pending);
    assert_eq!(submitted, TransactionStatus::Submitted);
    assert_eq!(confirmed, TransactionStatus::Confirmed);
    assert_eq!(finalized, TransactionStatus::Finalized);
    assert_eq!(expired, TransactionStatus::Expired);
    
    match failed {
        TransactionStatus::Failed(msg) => assert_eq!(msg, "InsufficientFunds"),
        _ => panic!("Expected Failed status"),
    }
}

/// Test: Transaction status clone
#[test]
fn test_transaction_status_clone() {
    let status1 = TransactionStatus::Confirmed;
    let status2 = status1.clone();
    
    assert_eq!(status1, status2);
}

/// Test: Failed transaction with different errors
#[test]
fn test_failed_transaction_errors() {
    let errors = vec![
        "InsufficientFunds",
        "AccountNotFound",
        "ProgramError",
        "InvalidSignature",
        "BlockhashNotFound",
    ];
    
    for error in errors {
        let failed = TransactionStatus::Failed(error.to_string());
        
        match failed {
            TransactionStatus::Failed(msg) => assert_eq!(msg, error),
            _ => panic!("Expected Failed status"),
        }
    }
}

/// Test: Route type variants
#[test]
fn test_route_type_variants() {
    let jito_bundle = RouteType::JitoBundle;
    let jito_single = RouteType::JitoSingle;
    let firedancer = RouteType::Firedancer;
    let standard = RouteType::StandardRpc;
    
    assert_eq!(jito_bundle, RouteType::JitoBundle);
    assert_eq!(jito_single, RouteType::JitoSingle);
    assert_eq!(firedancer, RouteType::Firedancer);
    assert_eq!(standard, RouteType::StandardRpc);
}

/// Test: Route type requires bundle
#[test]
fn test_route_type_requires_bundle() {
    assert!(RouteType::JitoBundle.requires_bundle());
    assert!(!RouteType::JitoSingle.requires_bundle());
    assert!(!RouteType::Firedancer.requires_bundle());
    assert!(!RouteType::StandardRpc.requires_bundle());
}

/// Test: Route type clone
#[test]
fn test_route_type_clone() {
    let route1 = RouteType::JitoBundle;
    let route2 = route1.clone();
    
    assert_eq!(route1, route2);
}

/// Test: MEV risk score serialization
#[test]
fn test_mev_risk_score_serialization() {
    let score = MevRiskScore::new(0.75);
    
    let json = serde_json::to_string(&score).unwrap();
    let deserialized: MevRiskScore = serde_json::from_str(&json).unwrap();
    
    assert_eq!(score.score(), deserialized.score());
}

/// Test: Transaction status serialization
#[test]
fn test_transaction_status_serialization() {
    let status = TransactionStatus::Confirmed;
    
    let json = serde_json::to_string(&status).unwrap();
    let deserialized: TransactionStatus = serde_json::from_str(&json).unwrap();
    
    assert_eq!(status, deserialized);
}

/// Test: Route type serialization
#[test]
fn test_route_type_serialization() {
    let route = RouteType::JitoBundle;
    
    let json = serde_json::to_string(&route).unwrap();
    let deserialized: RouteType = serde_json::from_str(&json).unwrap();
    
    assert_eq!(route, deserialized);
}

/// Test: MEV risk score ranges
#[test]
fn test_mev_risk_score_ranges() {
    let scores = vec![
        (0.0, true, false, false),    // low
        (0.25, true, false, false),   // low
        (0.49, true, false, false),   // low
        (0.5, false, true, false),    // medium
        (0.65, false, true, false),   // medium
        (0.79, false, true, false),   // medium
        (0.8, false, false, true),    // high
        (0.9, false, false, true),    // high
        (1.0, false, false, true),    // high
    ];
    
    for (value, is_low, is_medium, is_high) in scores {
        let score = MevRiskScore::new(value);
        assert_eq!(score.is_low_risk(), is_low, "Failed for score {}", value);
        assert_eq!(score.is_medium_risk(), is_medium, "Failed for score {}", value);
        assert_eq!(score.is_high_risk(), is_high, "Failed for score {}", value);
    }
}

/// Test: Multiple risk scores
#[test]
fn test_multiple_risk_scores() {
    let scores: Vec<MevRiskScore> = (0..=10)
        .map(|i| MevRiskScore::new(i as f32 / 10.0))
        .collect();
    
    assert_eq!(scores.len(), 11);
    assert_eq!(scores[0].score(), 0.0);
    assert_eq!(scores[10].score(), 1.0);
}

/// Test: Transaction status progression
#[test]
fn test_transaction_status_progression() {
    let statuses = [
        TransactionStatus::Pending,
        TransactionStatus::Submitted,
        TransactionStatus::Confirmed,
        TransactionStatus::Finalized,
    ];
    
    assert_eq!(statuses.len(), 4);
}

/// Test: Route type comparison
#[test]
fn test_route_type_comparison() {
    let route1 = RouteType::JitoBundle;
    let route2 = RouteType::JitoBundle;
    let route3 = RouteType::StandardRpc;
    
    assert_eq!(route1, route2);
    assert_ne!(route1, route3);
}

/// Test: MEV risk score copy trait
#[test]
fn test_mev_risk_score_copy() {
    let score1 = MevRiskScore::new(0.6);
    let score2 = score1; // Copy, not move
    
    assert_eq!(score1.score(), score2.score());
    // score1 is still accessible because it was copied
    assert_eq!(score1.score(), 0.6);
}

/// Test: Transaction status equality
#[test]
fn test_transaction_status_equality() {
    let status1 = TransactionStatus::Confirmed;
    let status2 = TransactionStatus::Confirmed;
    let status3 = TransactionStatus::Finalized;
    
    assert_eq!(status1, status2);
    assert_ne!(status1, status3);
}

/// Test: Failed status with different messages
#[test]
fn test_failed_status_inequality() {
    let failed1 = TransactionStatus::Failed("Error1".to_string());
    let failed2 = TransactionStatus::Failed("Error2".to_string());
    
    assert_ne!(failed1, failed2);
}

/// Test: Route type Eq trait
#[test]
fn test_route_type_eq() {
    let routes = vec![
        RouteType::JitoBundle,
        RouteType::JitoSingle,
        RouteType::Firedancer,
        RouteType::StandardRpc,
    ];
    
    for route in &routes {
        assert_eq!(route, route);
    }
}

/// Test: Risk score extreme values
#[test]
fn test_risk_score_extreme_values() {
    let very_negative = MevRiskScore::new(-1000.0);
    let very_positive = MevRiskScore::new(1000.0);
    
    assert_eq!(very_negative.score(), 0.0);
    assert_eq!(very_positive.score(), 1.0);
}

/// Test: Risk score precision
#[test]
fn test_risk_score_precision() {
    let precise_scores = vec![0.001, 0.123, 0.456, 0.789, 0.999];
    
    for value in precise_scores {
        let score = MevRiskScore::new(value);
        assert!((score.score() - value).abs() < 0.0001);
    }
}

/// Test: Transaction status debug output
#[test]
fn test_transaction_status_debug() {
    let status = TransactionStatus::Confirmed;
    let debug_str = format!("{:?}", status);
    
    assert!(debug_str.contains("Confirmed"));
}

/// Test: Route type debug output
#[test]
fn test_route_type_debug() {
    let route = RouteType::JitoBundle;
    let debug_str = format!("{:?}", route);
    
    assert!(debug_str.contains("JitoBundle"));
}

/// Test: MEV risk score debug output
#[test]
fn test_mev_risk_score_debug() {
    let score = MevRiskScore::new(0.85);
    let debug_str = format!("{:?}", score);
    
    assert!(debug_str.contains("MevRiskScore"));
}

/// Test: All route types don't require bundle except JitoBundle
#[test]
fn test_bundle_requirement_exclusivity() {
    let all_routes = [
        RouteType::JitoBundle,
        RouteType::JitoSingle,
        RouteType::Firedancer,
        RouteType::StandardRpc,
    ];
    
    let bundle_required_count = all_routes
        .iter()
        .filter(|r| r.requires_bundle())
        .count();
    
    assert_eq!(bundle_required_count, 1, "Only JitoBundle should require bundle");
}

/// Test: Risk score boundary precision
#[test]
fn test_risk_score_boundary_precision() {
    let scores = vec![
        (0.4999, true, false, false),   // just below 0.5
        (0.5, false, true, false),       // exactly 0.5
        (0.7999, false, true, false),   // just below 0.8
        (0.8, false, false, true),       // exactly 0.8 (is_high_risk)
    ];
    
    for (value, expected_low, expected_medium, expected_high) in scores {
        let score = MevRiskScore::new(value);
        assert_eq!(score.is_low_risk(), expected_low, "Low risk check failed for {}", value);
        assert_eq!(score.is_medium_risk(), expected_medium, "Medium risk check failed for {}", value);
        assert_eq!(score.is_high_risk(), expected_high, "High risk check failed for {}", value);
    }
}

/// Test: Transaction status with empty error message
#[test]
fn test_failed_status_empty_message() {
    let failed = TransactionStatus::Failed(String::new());
    
    match failed {
        TransactionStatus::Failed(msg) => assert_eq!(msg, ""),
        _ => panic!("Expected Failed status"),
    }
}

/// Test: Transaction status with long error message
#[test]
fn test_failed_status_long_message() {
    let long_error = "A".repeat(1000);
    let failed = TransactionStatus::Failed(long_error.clone());
    
    match failed {
        TransactionStatus::Failed(msg) => assert_eq!(msg.len(), 1000),
        _ => panic!("Expected Failed status"),
    }
}

/// Test: MEV risk score collection operations
#[test]
fn test_mev_risk_score_collection() {
    let scores: Vec<f32> = vec![0.1, 0.5, 0.9];
    let mev_scores: Vec<MevRiskScore> = scores
        .into_iter()
        .map(MevRiskScore::new)
        .collect();
    
    assert_eq!(mev_scores.len(), 3);
    assert!(mev_scores[0].is_low_risk());
    assert!(mev_scores[1].is_medium_risk());
    assert!(mev_scores[2].is_high_risk());
}

/// Test: Route type match patterns
#[test]
fn test_route_type_match_patterns() {
    let route = RouteType::JitoBundle;
    
    let description = match route {
        RouteType::JitoBundle => "Jito Bundle",
        RouteType::JitoSingle => "Jito Single",
        RouteType::Firedancer => "Firedancer",
        RouteType::StandardRpc => "Standard RPC",
    };
    
    assert_eq!(description, "Jito Bundle");
}

/// Test: Transaction status match patterns
#[test]
fn test_transaction_status_match_patterns() {
    let statuses = vec![
        (TransactionStatus::Pending, "pending"),
        (TransactionStatus::Submitted, "submitted"),
        (TransactionStatus::Confirmed, "confirmed"),
        (TransactionStatus::Finalized, "finalized"),
        (TransactionStatus::Failed("test".to_string()), "failed"),
        (TransactionStatus::Expired, "expired"),
    ];
    
    for (status, expected) in statuses {
        let result = match status {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Submitted => "submitted",
            TransactionStatus::Confirmed => "confirmed",
            TransactionStatus::Finalized => "finalized",
            TransactionStatus::Failed(_) => "failed",
            TransactionStatus::Expired => "expired",
        };
        
        assert_eq!(result, expected);
    }
}
