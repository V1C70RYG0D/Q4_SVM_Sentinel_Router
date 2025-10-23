use ai_engine::*;

#[test]
fn test_feature_vector_creation() {
    let _features = FeatureVector::default();
    assert!(FeatureVector::feature_count() > 0);
}

#[test]
fn test_inference_engine_creation() {
    // Use fallback engine (no model file required for tests)
    let result = InferenceEngine::fallback();
    assert!(result.is_ok());
}

#[test]
fn test_inference_requires_warmup() {
    let engine = InferenceEngine::fallback().expect("Failed to create engine");

    let features = FeatureVector::default();
    let result = engine.predict(&features);

    // Should fail without warmup
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not warmed up"));
}

#[test]
fn test_inference_after_warmup() {
    let mut engine = InferenceEngine::fallback().expect("Failed to create engine");

    // Perform warmup
    engine.warmup().expect("Warmup failed");

    let features = FeatureVector::default();
    let result = engine.predict(&features);

    // Should succeed after warmup
    assert!(result.is_ok());

    let risk_score = result.unwrap();
    assert!(risk_score.0 >= 0.0 && risk_score.0 <= 1.0);
}

#[test]
fn test_model_info() {
    let engine = InferenceEngine::fallback().expect("Failed to create engine");

    let info = engine.model_info();
    assert!(!info.warmup_complete);
    assert!(info.feature_count > 0);
}
