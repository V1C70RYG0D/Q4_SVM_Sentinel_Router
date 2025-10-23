use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ai_engine::{FeatureExtractor, FeatureVector, InferenceEngine, TransactionData, SwapDetailsData};
use solana_sdk::pubkey::Pubkey;

fn bench_feature_extraction(c: &mut Criterion) {
    let mut extractor = FeatureExtractor::new();
    
    let tx_data = TransactionData {
        slot: 100_000,
        fee_payer: Pubkey::new_unique(),
        compute_unit_limit: 200_000,
        compute_unit_price: 5_000,
        jito_tip_lamports: 50_000,
        total_fee_lamports: 100_000,
        account_count: 10,
        instruction_count: 5,
        tx_size_bytes: 1_500,
        swap_details: Some(SwapDetailsData {
            input_mint: Pubkey::new_unique(),
            output_mint: Pubkey::new_unique(),
            input_amount: 1_000_000.0,
            output_amount: 990_000.0,
            expected_output: 995_000.0,
            route_length: 2,
            slippage_tolerance_bps: 50.0,
            pool_liquidity_usd: 10_000_000.0,
        }),
        time_since_last_slot_ms: 400,
        next_leader_pubkey: Pubkey::new_unique(),
        uses_lookup_tables: false,
        timestamp_ms: 1_700_000_000_000,
    };
    
    c.bench_function("feature_extraction", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                black_box(extractor.extract(black_box(&tx_data)).await)
            })
        })
    });
}

fn bench_inference_prediction(c: &mut Criterion) {
    let mut engine = InferenceEngine::fallback().unwrap();
    let _ = engine.warmup();
    
    let features = FeatureVector::default();
    
    c.bench_function("inference_predict", |b| {
        b.iter(|| {
            black_box(engine.predict(black_box(&features)))
        })
    });
}

fn bench_feature_to_array(c: &mut Criterion) {
    let features = FeatureVector::default();
    
    c.bench_function("feature_to_array", |b| {
        b.iter(|| {
            black_box(features.to_array())
        })
    });
}

fn bench_feature_validation(c: &mut Criterion) {
    let features = FeatureVector::default();
    
    c.bench_function("feature_validation", |b| {
        b.iter(|| {
            black_box(features.validate())
        })
    });
}

fn bench_different_risk_levels(c: &mut Criterion) {
    let mut engine = InferenceEngine::fallback().unwrap();
    let _ = engine.warmup();
    
    let mut group = c.benchmark_group("risk_levels");
    
    // Low risk
    let low_risk = FeatureVector::default();
    group.bench_with_input(BenchmarkId::new("predict", "low_risk"), &low_risk, |b, f| {
        b.iter(|| engine.predict(black_box(f)))
    });
    
    // High risk
    let high_risk = FeatureVector {
        jito_tip_lamports: 200_000,
        has_swap_triplet: true,
        next_leader_malicious: true,
        ..Default::default()
    };
    group.bench_with_input(BenchmarkId::new("predict", "high_risk"), &high_risk, |b, f| {
        b.iter(|| engine.predict(black_box(f)))
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_feature_extraction,
    bench_inference_prediction,
    bench_feature_to_array,
    bench_feature_validation,
    bench_different_risk_levels
);

criterion_main!(benches);
