//! Benchmarks for Intent validation and hashing performance
//!
//! Target SLO: <5ms for intent validation

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sentinel_core::{
    ConsentBlock, Constraints, FeePreferences, Intent, IntentType, SwapDetails, SwapMode,
};
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use chrono::Utc;

fn create_benchmark_swap_intent() -> Intent {
    Intent {
        intent_id: Intent::new_signature_request_id(),
        user_public_key: Pubkey::new_unique(),
        intent_type: IntentType::Swap,
        swap_details: Some(SwapDetails {
            mode: SwapMode::ExactIn,
            input_mint: Pubkey::new_unique(),
            output_mint: Pubkey::new_unique(),
            amount: 1_000_000_000,
            minimum_received: Some(900_000_000),
            dex: Some("Jupiter".to_string()),
            route_hints: Some(vec![Pubkey::new_unique(), Pubkey::new_unique()]),
        }),
        constraints: Constraints {
            max_slippage_bps: 50,
            partial_fill: false,
            expiry_timestamp: Some(Utc::now().timestamp() + 3600),
            ttl_seconds: None,
        },
        fee_preferences: FeePreferences {
            max_priority_fee_lamports: 100_000,
            max_jito_tip_lamports: 50_000,
            tip_allocation_pct: 70,
        },
        consent_block: ConsentBlock {
            recent_blockhash: Hash::new_unique(),
            signature_request_id: Intent::new_signature_request_id(),
            nonce: Some(Hash::new_unique().to_string()),
        },
        limit_details: None,
        twap_details: None,
    }
}

fn bench_intent_validation(c: &mut Criterion) {
    let intent = create_benchmark_swap_intent();
    let current_time = Utc::now().timestamp();

    c.bench_function("intent_validate", |b| {
        b.iter(|| {
            black_box(&intent).validate(black_box(current_time)).ok();
        });
    });
}

fn bench_intent_hashing(c: &mut Criterion) {
    let intent = create_benchmark_swap_intent();

    c.bench_function("intent_hash", |b| {
        b.iter(|| {
            black_box(&intent).hash();
        });
    });
}

fn bench_intent_priority_level(c: &mut Criterion) {
    let intent = create_benchmark_swap_intent();

    c.bench_function("intent_priority_level", |b| {
        b.iter(|| {
            black_box(&intent).priority_level();
        });
    });
}

fn bench_json_serialization(c: &mut Criterion) {
    let intent = create_benchmark_swap_intent();

    c.bench_function("intent_json_serialize", |b| {
        b.iter(|| {
            serde_json::to_string(black_box(&intent)).unwrap();
        });
    });
}

fn bench_json_deserialization(c: &mut Criterion) {
    let intent = create_benchmark_swap_intent();
    let json = serde_json::to_string(&intent).unwrap();

    c.bench_function("intent_json_deserialize", |b| {
        b.iter(|| {
            serde_json::from_str::<Intent>(black_box(&json)).unwrap();
        });
    });
}

fn bench_bincode_serialization(c: &mut Criterion) {
    let intent = create_benchmark_swap_intent();

    c.bench_function("intent_bincode_serialize", |b| {
        b.iter(|| {
            bincode::serialize(black_box(&intent)).unwrap();
        });
    });
}

fn bench_bincode_deserialization(c: &mut Criterion) {
    let intent = create_benchmark_swap_intent();
    let encoded = bincode::serialize(&intent).unwrap();

    c.bench_function("intent_bincode_deserialize", |b| {
        b.iter(|| {
            bincode::deserialize::<Intent>(black_box(&encoded)).unwrap();
        });
    });
}

fn bench_full_intent_pipeline(c: &mut Criterion) {
    c.bench_function("intent_full_pipeline", |b| {
        b.iter(|| {
            let intent = create_benchmark_swap_intent();
            let current_time = Utc::now().timestamp();
            
            // Validate
            intent.validate(current_time).ok();
            
            // Compute hash
            let _hash = intent.hash();
            
            // Get priority
            let _priority = intent.priority_level();
            
            // Serialize to JSON
            let _json = serde_json::to_string(&intent).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_intent_validation,
    bench_intent_hashing,
    bench_intent_priority_level,
    bench_json_serialization,
    bench_json_deserialization,
    bench_bincode_serialization,
    bench_bincode_deserialization,
    bench_full_intent_pipeline,
);
criterion_main!(benches);
