use chrono::Utc;
use sentinel_core::{
    ConsentBlock, Constraints, FeePreferences, Intent, IntentType, MevRiskScore, RouteType, SwapDetails,
    SwapMode, TransactionStatus,
};
use solana_sdk::{hash::Hash, pubkey::Pubkey};

#[test]
fn test_intent_creation() {
    let user_pubkey = Pubkey::new_unique();
    let intent = Intent {
        intent_id: "test-intent-123".to_string(),
        user_public_key: user_pubkey,
        intent_type: IntentType::Swap,
        swap_details: Some(SwapDetails {
            input_mint: Pubkey::new_unique(),
            output_mint: Pubkey::new_unique(),
            amount: 1_000_000_000, // 1 SOL
            mode: SwapMode::ExactIn,
            minimum_received: None,
            dex: None,
            route_hints: None,
        }),
        constraints: Constraints {
            max_slippage_bps: 100, // 1%
            expiry_timestamp: Some(Utc::now().timestamp() + 60),
            partial_fill: false,
            ttl_seconds: None,
        },
        fee_preferences: FeePreferences {
            max_priority_fee_lamports: 10_000,
            max_jito_tip_lamports: 100_000,
            tip_allocation_pct: 50, // 50% of tip goes to priority
        },
        consent_block: ConsentBlock {
            recent_blockhash: Hash::default(),
            signature_request_id: "test-sig-req-123".to_string(),
            nonce: None,
        },
        limit_details: None,
        twap_details: None,
    };

    assert!(matches!(intent.intent_type, IntentType::Swap));
    assert!(intent.swap_details.is_some());
}

#[test]
fn test_mev_risk_score() {
    let low_risk = MevRiskScore::new(0.1);
    let high_risk = MevRiskScore::new(0.85);

    assert!(!low_risk.is_high_risk());
    assert!(high_risk.is_high_risk());
    
    // Test medium risk
    let medium_risk = MevRiskScore::new(0.6);
    assert!(medium_risk.is_medium_risk());
    assert!(!medium_risk.is_high_risk());
    assert!(!medium_risk.is_low_risk());
    
    // Test low risk
    assert!(low_risk.is_low_risk());
    assert!(!low_risk.is_medium_risk());
    
    // Test score clamping
    let clamped_high = MevRiskScore::new(1.5);
    assert_eq!(clamped_high.score(), 1.0);
    
    let clamped_low = MevRiskScore::new(-0.5);
    assert_eq!(clamped_low.score(), 0.0);
    
    // Test boundary conditions
    let boundary_medium = MevRiskScore::new(0.5);
    assert!(boundary_medium.is_medium_risk());
    assert!(!boundary_medium.is_low_risk());
    
    let boundary_high = MevRiskScore::new(0.8);
    assert!(boundary_high.is_high_risk());
    assert!(!boundary_high.is_medium_risk());
}

#[test]
fn test_transaction_status() {
    let status = TransactionStatus::Pending;
    assert_eq!(format!("{:?}", status), "Pending");
    
    // Test all status variants
    let submitted = TransactionStatus::Submitted;
    assert_eq!(format!("{:?}", submitted), "Submitted");
    
    let confirmed = TransactionStatus::Confirmed;
    assert_eq!(format!("{:?}", confirmed), "Confirmed");
    
    let finalized = TransactionStatus::Finalized;
    assert_eq!(format!("{:?}", finalized), "Finalized");
    
    let failed = TransactionStatus::Failed("timeout".to_string());
    assert!(matches!(failed, TransactionStatus::Failed(_)));
    
    let expired = TransactionStatus::Expired;
    assert_eq!(format!("{:?}", expired), "Expired");
    
    // Test equality
    assert_eq!(TransactionStatus::Pending, TransactionStatus::Pending);
    assert_ne!(TransactionStatus::Pending, TransactionStatus::Confirmed);
}

#[test]
fn test_route_type() {
    // Test requires_bundle for all variants
    assert!(RouteType::JitoBundle.requires_bundle());
    assert!(!RouteType::JitoSingle.requires_bundle());
    assert!(!RouteType::Firedancer.requires_bundle());
    assert!(!RouteType::StandardRpc.requires_bundle());
    
    // Test equality
    assert_eq!(RouteType::JitoBundle, RouteType::JitoBundle);
    assert_ne!(RouteType::JitoBundle, RouteType::JitoSingle);
}
