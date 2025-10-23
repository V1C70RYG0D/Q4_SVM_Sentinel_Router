use ai_engine::model::ModelConfig;
use std::path::PathBuf;

#[test]
fn test_model_config_default() {
    let config = ModelConfig::default();
    assert_eq!(config.model_path, PathBuf::from("models/mev_detector.onnx"));
    assert_eq!(config.intra_op_threads, 4);
    assert_eq!(config.inter_op_threads, 1);
    assert_eq!(config.warmup_iterations, 100);
    assert!(config.enable_quantization);
    assert!(config.enable_memory_pattern);
    assert_eq!(config.graph_optimization_level, 3);
    assert!(config.enable_parallel_execution);
}

#[test]
fn test_model_config_new() {
    let custom_path = PathBuf::from("custom/model.onnx");
    let config = ModelConfig::new(custom_path.clone());
    assert_eq!(config.model_path, custom_path);
    // Other fields should be default
    assert_eq!(config.intra_op_threads, 4);
    assert!(config.enable_quantization);
}

#[test]
fn test_model_config_with_threads() {
    let config = ModelConfig::default()
        .with_threads(8, 2);
    assert_eq!(config.intra_op_threads, 8);
    assert_eq!(config.inter_op_threads, 2);
}

#[test]
fn test_model_config_with_warmup() {
    let config = ModelConfig::default()
        .with_warmup(200);
    assert_eq!(config.warmup_iterations, 200);
}

#[test]
fn test_model_config_with_onnx_optimizations_enabled() {
    let config = ModelConfig::default()
        .with_onnx_optimizations(true);
    assert!(config.enable_memory_pattern);
    assert_eq!(config.graph_optimization_level, 3);
    assert!(config.enable_parallel_execution);
}

#[test]
fn test_model_config_with_onnx_optimizations_disabled() {
    let config = ModelConfig::default()
        .with_onnx_optimizations(false);
    assert!(!config.enable_memory_pattern);
    assert_eq!(config.graph_optimization_level, 0);
    assert!(!config.enable_parallel_execution);
}

#[test]
fn test_model_config_no_optimizations() {
    let config = ModelConfig::default()
        .no_optimizations();
    assert!(!config.enable_memory_pattern);
    assert_eq!(config.graph_optimization_level, 0);
    assert!(!config.enable_parallel_execution);
}

#[test]
fn test_model_config_builder_chain() {
    let config = ModelConfig::new(PathBuf::from("test.onnx"))
        .with_threads(16, 4)
        .with_warmup(50)
        .with_onnx_optimizations(true);
    
    assert_eq!(config.model_path, PathBuf::from("test.onnx"));
    assert_eq!(config.intra_op_threads, 16);
    assert_eq!(config.inter_op_threads, 4);
    assert_eq!(config.warmup_iterations, 50);
    assert!(config.enable_memory_pattern);
    assert_eq!(config.graph_optimization_level, 3);
}

#[test]
fn test_model_config_serialization() {
    let config = ModelConfig::default();
    
    // Test that it can be serialized to JSON
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("model_path"));
    assert!(json.contains("intra_op_threads"));
    
    // Test that it can be deserialized from JSON
    let deserialized: ModelConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.intra_op_threads, config.intra_op_threads);
    assert_eq!(deserialized.enable_memory_pattern, config.enable_memory_pattern);
}
