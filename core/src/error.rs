use thiserror::Error;

#[derive(Error, Debug)]
pub enum SentinelError {
    #[error("Invalid intent: {0}")]
    InvalidIntent(String),

    #[error("Intent validation error: {0}")]
    IntentValidation(#[from] crate::intent::IntentError),

    #[error("Ingestion error: {0}")]
    IngestionError(String),

    #[error("AI inference error: {0}")]
    InferenceError(String),

    #[error("Bundle construction error: {0}")]
    BundleError(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Price oracle error: {0}")]
    PriceOracleError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("DEX error: {0}")]
    DexError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, SentinelError>;
