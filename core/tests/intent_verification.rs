//! Comprehensive Intent Tests
//!
//! Tests cover: validation logic, future intent types, error handling,
//! JSON/bincode roundtrips, and edge cases

use chrono::Utc;
use sentinel_core::{
    ConsentBlock, Constraints, FeePreferences, Intent, IntentError, IntentType, LimitDetails,
    Priority, SwapDetails, SwapMode, TwapDetails,
};
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;

fn create_valid_swap_intent() -> Intent {
    Intent {
        intent_id: Intent::new_signature_request_id(),
        user_public_key: Pubkey::new_unique(),
        intent_type: IntentType::Swap,
        swap_details: Some(SwapDetails {
            mode: SwapMode::ExactIn,
            input_mint: Pubkey::new_unique(),
            output_mint: Pubkey::new_unique(),
            amount: 1_000_000,
            minimum_received: Some(900_000),
            dex: Some("Jupiter".to_string()),
            route_hints: None,
        }),
        constraints: Constraints::default(),
        fee_preferences: FeePreferences::default(),
        consent_block: ConsentBlock {
            recent_blockhash: Hash::new_unique(),
            signature_request_id: Intent::new_signature_request_id(),
            nonce: None,
        },
        limit_details: None,
        twap_details: None,
    }
}

// ================================================================================================
// Validation Tests
// ================================================================================================

#[test]
fn test_valid_swap_intent() {
    let intent = create_valid_swap_intent();
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_missing_swap_details() {
    let mut intent = create_valid_swap_intent();
    intent.swap_details = None;
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::MissingSwapDetails)
    );
}

#[test]
fn test_zero_amount() {
    let mut intent = create_valid_swap_intent();
    intent.swap_details.as_mut().unwrap().amount = 0;
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidAmount)
    );
}

#[test]
fn test_same_mints() {
    let mut intent = create_valid_swap_intent();
    let same_mint = Pubkey::new_unique();
    let swap = intent.swap_details.as_mut().unwrap();
    swap.input_mint = same_mint;
    swap.output_mint = same_mint;
    let current_time = Utc::now().timestamp();
    assert_eq!(intent.validate(current_time), Err(IntentError::SameMints));
}

#[test]
fn test_slippage_too_high() {
    let mut intent = create_valid_swap_intent();
    intent.constraints.max_slippage_bps = 10001;
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::SlippageTooHigh)
    );
}

#[test]
fn test_zero_fees() {
    let mut intent = create_valid_swap_intent();
    intent.fee_preferences.max_priority_fee_lamports = 0;
    intent.fee_preferences.max_jito_tip_lamports = 0;
    let current_time = Utc::now().timestamp();
    assert_eq!(intent.validate(current_time), Err(IntentError::ZeroFees));
}

#[test]
fn test_invalid_tip_allocation() {
    let mut intent = create_valid_swap_intent();
    intent.fee_preferences.tip_allocation_pct = 101;
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidTipAllocation)
    );
}

#[test]
fn test_valid_tip_allocation_boundary() {
    let mut intent = create_valid_swap_intent();
    intent.fee_preferences.tip_allocation_pct = 100;
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_expired_intent() {
    let mut intent = create_valid_swap_intent();
    let current_time = Utc::now().timestamp();
    intent.constraints.expiry_timestamp = Some(current_time - 10);
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidExpiry(30))
    );
}

#[test]
fn test_expiry_with_buffer() {
    let mut intent = create_valid_swap_intent();
    let current_time = Utc::now().timestamp();
    
    // Expiry too close (within buffer)
    intent.constraints.expiry_timestamp = Some(current_time + 10);
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidExpiry(30))
    );

    // Valid expiry (beyond buffer)
    intent.constraints.expiry_timestamp = Some(current_time + 60);
    assert!(intent.validate(current_time).is_ok());
}

// ================================================================================================
// Future Intent Types Tests
// ================================================================================================

#[test]
fn test_limit_intent_missing_details() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::Limit;
    intent.limit_details = None;
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::MissingLimitDetails)
    );
}

#[test]
fn test_limit_intent_invalid_threshold() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::Limit;
    intent.limit_details = Some(LimitDetails {
        price_threshold: 0.0,
        oracle: None,
    });
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidPriceThreshold)
    );
}

#[test]
fn test_limit_intent_negative_threshold() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::Limit;
    intent.limit_details = Some(LimitDetails {
        price_threshold: -1.5,
        oracle: None,
    });
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidPriceThreshold)
    );
}

#[test]
fn test_limit_intent_unimplemented() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::Limit;
    intent.limit_details = Some(LimitDetails {
        price_threshold: 1.5,
        oracle: None,
    });
    let current_time = Utc::now().timestamp();
    // Now that we have real validation, valid limit intents should pass
    assert_eq!(intent.validate(current_time), Ok(()));
}

#[test]
fn test_twap_intent_missing_details() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::TWAP;
    intent.twap_details = None;
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::MissingTwapDetails)
    );
}

#[test]
fn test_twap_intent_zero_duration() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::TWAP;
    intent.twap_details = Some(TwapDetails {
        duration_secs: 0,
        num_chunks: Some(10),
    });
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidTwapDuration)
    );
}

#[test]
fn test_twap_intent_unimplemented() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::TWAP;
    intent.twap_details = Some(TwapDetails {
        duration_secs: 3600,
        num_chunks: Some(10),
    });
    let current_time = Utc::now().timestamp();
    // Now that we have real validation, valid TWAP intents should pass
    assert_eq!(intent.validate(current_time), Ok(()));
}

#[test]
fn test_twap_intent_invalid_duration_too_short() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::TWAP;
    intent.twap_details = Some(TwapDetails {
        duration_secs: 30, // Less than 60 seconds minimum
        num_chunks: Some(5),
    });
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidTwapDuration)
    );
}

#[test]
fn test_twap_intent_invalid_duration_too_long() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::TWAP;
    intent.twap_details = Some(TwapDetails {
        duration_secs: 90000, // More than 86400 seconds (24 hours) maximum
        num_chunks: Some(10),
    });
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidTwapDuration)
    );
}

#[test]
fn test_limit_intent_invalid_astronomical_threshold() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::Limit;
    intent.limit_details = Some(LimitDetails {
        price_threshold: 2e18, // Astronomical value > 1e18
        oracle: None,
    });
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidPriceThreshold)
    );
}

#[test]
fn test_limit_intent_invalid_infinite_threshold() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::Limit;
    intent.limit_details = Some(LimitDetails {
        price_threshold: f64::INFINITY,
        oracle: None,
    });
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidPriceThreshold)
    );
}

#[test]
fn test_limit_intent_invalid_nan_threshold() {
    let mut intent = create_valid_swap_intent();
    intent.intent_type = IntentType::Limit;
    intent.limit_details = Some(LimitDetails {
        price_threshold: f64::NAN,
        oracle: None,
    });
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidPriceThreshold)
    );
}

// ================================================================================================
// Nonce Tests
// ================================================================================================

#[test]
fn test_invalid_nonce_format() {
    let mut intent = create_valid_swap_intent();
    intent.consent_block.nonce = Some("invalid_base58!!!".to_string());
    let current_time = Utc::now().timestamp();
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidNonce)
    );
}

#[test]
fn test_valid_nonce() {
    let mut intent = create_valid_swap_intent();
    let valid_hash = Hash::new_unique();
    intent.consent_block.nonce = Some(valid_hash.to_string());
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_none_nonce() {
    let mut intent = create_valid_swap_intent();
    intent.consent_block.nonce = None;
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

// ================================================================================================
// Priority Tests
// ================================================================================================

#[test]
fn test_priority_low() {
    let mut intent = create_valid_swap_intent();
    intent.fee_preferences.max_priority_fee_lamports = 5_000;
    intent.fee_preferences.max_jito_tip_lamports = 0;
    assert_eq!(intent.priority_level(), Priority::Low);
}

#[test]
fn test_priority_medium() {
    let mut intent = create_valid_swap_intent();
    intent.fee_preferences.max_priority_fee_lamports = 30_000;
    intent.fee_preferences.max_jito_tip_lamports = 0;
    assert_eq!(intent.priority_level(), Priority::Medium);
}

#[test]
fn test_priority_high() {
    let mut intent = create_valid_swap_intent();
    intent.fee_preferences.max_priority_fee_lamports = 100_000;
    intent.fee_preferences.max_jito_tip_lamports = 0;
    assert_eq!(intent.priority_level(), Priority::High);
}

#[test]
fn test_priority_critical() {
    let mut intent = create_valid_swap_intent();
    intent.fee_preferences.max_priority_fee_lamports = 200_000;
    intent.fee_preferences.max_jito_tip_lamports = 100_000;
    assert_eq!(intent.priority_level(), Priority::Critical);
}

#[test]
fn test_priority_boundary_low_to_medium() {
    let mut intent = create_valid_swap_intent();
    intent.fee_preferences.max_priority_fee_lamports = 10_000;
    intent.fee_preferences.max_jito_tip_lamports = 0;
    assert_eq!(intent.priority_level(), Priority::Low);

    intent.fee_preferences.max_priority_fee_lamports = 10_001;
    assert_eq!(intent.priority_level(), Priority::Medium);
}

// ================================================================================================
// Hashing Tests
// ================================================================================================

#[test]
fn test_intent_hashing_deterministic() {
    let intent = create_valid_swap_intent();
    let hash1 = intent.hash();
    let hash2 = intent.hash();
    assert_eq!(hash1, hash2);
}

#[test]
fn test_intent_hashing_different_content() {
    let intent1 = create_valid_swap_intent();
    let intent2 = create_valid_swap_intent();
    
    let hash1 = intent1.hash();
    let hash3 = intent2.hash();
    assert_ne!(hash1, hash3);
}

#[test]
fn test_intent_hashing_tampering_detection() {
    let mut intent1 = create_valid_swap_intent();
    let hash1 = intent1.hash();
    
    // Modify amount
    intent1.swap_details.as_mut().unwrap().amount = 999_999;
    let hash2 = intent1.hash();
    
    assert_ne!(hash1, hash2, "Hash should change when intent is modified");
}

// ================================================================================================
// Serialization Tests
// ================================================================================================

#[test]
fn test_json_serialization_roundtrip() {
    let intent = create_valid_swap_intent();
    let json = serde_json::to_string(&intent).expect("Serialization failed");
    
    // Verify JSON contains expected fields
    assert!(json.contains("intent_type"));
    assert!(json.contains("swap"));
    assert!(json.contains("constraints"));
    assert!(json.contains("consent_block"));
    
    let deserialized: Intent =
        serde_json::from_str(&json).expect("Deserialization failed");
    
    // Verify critical fields match
    assert_eq!(deserialized.intent_type, intent.intent_type);
    assert_eq!(
        deserialized.swap_details.as_ref().unwrap().amount,
        intent.swap_details.as_ref().unwrap().amount
    );
    assert_eq!(
        deserialized.constraints.max_slippage_bps,
        intent.constraints.max_slippage_bps
    );
}

#[test]
fn test_json_deserialization_invalid() {
    let invalid_json = r#"{"intent_type": "invalid_type"}"#;
    let result = serde_json::from_str::<Intent>(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_bincode_serialization_roundtrip() {
    let intent = create_valid_swap_intent();
    let encoded = bincode::serialize(&intent)
        .expect("Encoding failed");
    
    let decoded: Intent = bincode::deserialize(&encoded)
        .expect("Decoding failed");
    
    assert_eq!(decoded, intent);
}

#[test]
fn test_bincode_smaller_than_json() {
    let intent = create_valid_swap_intent();
    
    let json = serde_json::to_string(&intent).unwrap();
    let bincode_data = bincode::serialize(&intent).unwrap();
    
    // Bincode should be more compact
    assert!(
        bincode_data.len() < json.len(),
        "Bincode size: {}, JSON size: {}",
        bincode_data.len(),
        json.len()
    );
}

#[test]
fn test_hash_serialization_as_base58() {
    let intent = create_valid_swap_intent();
    let json = serde_json::to_string(&intent).unwrap();
    
    // Verify hash is serialized as base58 string, not bytes
    assert!(json.contains(&intent.consent_block.recent_blockhash.to_string()));
}

// ================================================================================================
// Defaults Tests
// ================================================================================================

#[test]
fn test_constraints_defaults() {
    let constraints = Constraints::default();
    assert_eq!(constraints.max_slippage_bps, 50);
    assert!(!constraints.partial_fill);
    assert!(constraints.expiry_timestamp.is_none());
}

#[test]
fn test_fee_preferences_defaults() {
    let fee_prefs = FeePreferences::default();
    assert_eq!(fee_prefs.max_priority_fee_lamports, 100_000);
    assert_eq!(fee_prefs.max_jito_tip_lamports, 50_000);
    assert_eq!(fee_prefs.tip_allocation_pct, 70);
}

// ================================================================================================
// Edge Cases
// ================================================================================================

#[test]
fn test_exact_out_swap_mode() {
    let mut intent = create_valid_swap_intent();
    intent.swap_details.as_mut().unwrap().mode = SwapMode::ExactOut;
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_very_large_amount() {
    let mut intent = create_valid_swap_intent();
    intent.swap_details.as_mut().unwrap().amount = u64::MAX;
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_maximum_slippage() {
    let mut intent = create_valid_swap_intent();
    intent.constraints.max_slippage_bps = 10000; // 100%
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_zero_slippage() {
    let mut intent = create_valid_swap_intent();
    intent.constraints.max_slippage_bps = 0;
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_partial_fill_enabled() {
    let mut intent = create_valid_swap_intent();
    intent.constraints.partial_fill = true;
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_no_dex_preference() {
    let mut intent = create_valid_swap_intent();
    intent.swap_details.as_mut().unwrap().dex = None;
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_with_route_hints() {
    let mut intent = create_valid_swap_intent();
    intent.swap_details.as_mut().unwrap().route_hints = Some(vec![
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
    ]);
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_signature_request_id_uniqueness() {
    let id1 = Intent::new_signature_request_id();
    let id2 = Intent::new_signature_request_id();
    assert_ne!(id1, id2, "Signature request IDs should be unique");
}

#[test]
fn test_signature_request_id_format() {
    let id = Intent::new_signature_request_id();
    // UUID v4 format: 8-4-4-4-12 hexadecimal characters
    assert_eq!(id.len(), 36);
    assert_eq!(id.chars().filter(|&c| c == '-').count(), 4);
}

// ================================================================================================
// Integration Tests
// ================================================================================================

#[test]
fn test_complete_intent_lifecycle() {
    // Create intent
    let intent = create_valid_swap_intent();
    let current_time = Utc::now().timestamp();
    
    // Validate
    assert!(intent.validate(current_time).is_ok());
    
    // Compute hash
    let hash = intent.hash();
    assert_ne!(hash, Hash::default());
    
    // Check priority (defaults: 100k + 50k = 150k lamports = High)
    let priority = intent.priority_level();
    assert_eq!(priority, Priority::High);
    
    // Serialize to JSON
    let json = serde_json::to_string(&intent).unwrap();
    
    // Deserialize from JSON
    let deserialized: Intent = serde_json::from_str(&json).unwrap();
    
    // Validate deserialized
    assert!(deserialized.validate(current_time).is_ok());
    
    // Hash should match
    assert_eq!(deserialized.hash(), hash);
}

#[test]
fn test_intent_with_all_optional_fields() {
    let intent = Intent {
        intent_id: Intent::new_signature_request_id(),
        user_public_key: Pubkey::new_unique(),
        intent_type: IntentType::Swap,
        swap_details: Some(SwapDetails {
            mode: SwapMode::ExactIn,
            input_mint: Pubkey::new_unique(),
            output_mint: Pubkey::new_unique(),
            amount: 1_000_000,
            minimum_received: Some(900_000),
            dex: Some("Jupiter".to_string()),
            route_hints: Some(vec![Pubkey::new_unique()]),
        }),
        constraints: Constraints {
            max_slippage_bps: 100,
            partial_fill: true,
            expiry_timestamp: Some(Utc::now().timestamp() + 3600),
            ttl_seconds: None,
        },
        fee_preferences: FeePreferences {
            max_priority_fee_lamports: 200_000,
            max_jito_tip_lamports: 100_000,
            tip_allocation_pct: 80,
        },
        consent_block: ConsentBlock {
            recent_blockhash: Hash::new_unique(),
            signature_request_id: Intent::new_signature_request_id(),
            nonce: Some(Hash::new_unique().to_string()),
        },
        limit_details: None,
        twap_details: None,
    };
    
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_performance_target_validation() {
    use std::time::Instant;
    
    let intent = create_valid_swap_intent();
    let current_time = Utc::now().timestamp();
    
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = intent.validate(current_time);
    }
    let duration = start.elapsed();
    
    let avg_duration = duration.as_micros() / 1000;
    
    // Target: <5ms per validation (5000 microseconds)
    assert!(
        avg_duration < 5000,
        "Average validation time: {}μs (target: <5000μs)",
        avg_duration
    );
}

// ================================================================================================
// Future Enhancement Tests (Q1 2026)
// ================================================================================================

#[test]
fn test_limit_order_with_oracle() {
    let intent = Intent {
        intent_id: Intent::new_signature_request_id(),
        user_public_key: Pubkey::new_unique(),
        intent_type: IntentType::Limit,
        swap_details: None,
        constraints: Constraints::default(),
        fee_preferences: FeePreferences::default(),
        consent_block: ConsentBlock {
            recent_blockhash: Hash::new_unique(),
            signature_request_id: Intent::new_signature_request_id(),
            nonce: None,
        },
        limit_details: Some(LimitDetails {
            price_threshold: 100.5,
            oracle: Some(Pubkey::new_unique()), // Pyth oracle address
        }),
        twap_details: None,
    };
    
    // Limit orders now validate successfully with real validation logic
    let current_time = Utc::now().timestamp();
    assert_eq!(intent.validate(current_time), Ok(()));
}

#[test]
fn test_ttl_expiration() {
    let mut intent = create_valid_swap_intent();
    
    // Set TTL to 60 seconds (valid)
    intent.constraints.ttl_seconds = Some(60);
    let current_time = Utc::now().timestamp();
    assert!(intent.validate(current_time).is_ok());
    
    // Set TTL too short (< 30 second buffer)
    intent.constraints.ttl_seconds = Some(15);
    assert_eq!(
        intent.validate(current_time),
        Err(IntentError::InvalidExpiry(30))
    );
}

#[test]
fn test_ttl_vs_expiry_timestamp_precedence() {
    let mut intent = create_valid_swap_intent();
    let current_time = Utc::now().timestamp();
    
    // Set both TTL and expiry_timestamp
    intent.constraints.ttl_seconds = Some(10); // Too short
    intent.constraints.expiry_timestamp = Some(current_time + 300); // Valid
    
    // expiry_timestamp takes precedence, so should be valid
    assert!(intent.validate(current_time).is_ok());
}

#[test]
fn test_intent_status_serialization() {
    use sentinel_core::IntentStatus;
    
    // Test all status variants serialize correctly
    let statuses = vec![
        IntentStatus::Pending,
        IntentStatus::Submitted,
        IntentStatus::Confirmed,
        IntentStatus::Failed("Transaction timeout".to_string()),
        IntentStatus::Expired,
    ];
    
    for status in statuses {
        let json = serde_json::to_string(&status).expect("Serialization failed");
        let deserialized: IntentStatus = 
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(status, deserialized);
    }
}

#[test]
fn test_intent_status_monitoring_workflow() {
    use sentinel_core::IntentStatus;
    
    // Simulate intent lifecycle
    let mut status = IntentStatus::Pending;
    assert_eq!(status, IntentStatus::Pending);
    
    // Submit to network
    status = IntentStatus::Submitted;
    assert_eq!(status, IntentStatus::Submitted);
    
    // Confirm on-chain
    status = IntentStatus::Confirmed;
    assert_eq!(status, IntentStatus::Confirmed);
}

#[test]
fn test_constraints_default_includes_ttl() {
    let constraints = Constraints::default();
    assert_eq!(constraints.max_slippage_bps, 50);
    assert!(!constraints.partial_fill);
    assert_eq!(constraints.expiry_timestamp, None);
    assert_eq!(constraints.ttl_seconds, None);
}
