//! Database error types and result aliases

use thiserror::Error;

/// Database-specific error types
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Query execution failed: {0}")]
    QueryFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Time range error: {0}")]
    TimeRangeError(String),

    #[error("Sensor not found: {0}")]
    SensorNotFound(String),

    #[error("Invalid data format: {0}")]
    InvalidDataFormat(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Connection timeout after {timeout_secs} seconds")]
    ConnectionTimeout { timeout_secs: u64 },

    #[error("Query timeout after {timeout_secs} seconds")]
    QueryTimeout { timeout_secs: u64 },

    #[error("Database not available: {reason}")]
    DatabaseUnavailable { reason: String },

    #[error("Feature not supported by backend: {feature}")]
    FeatureNotSupported { feature: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result type alias for database operations
pub type DatabaseResult<T> = Result<T, DatabaseError>;

impl DatabaseError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            DatabaseError::ConnectionTimeout { .. }
                | DatabaseError::QueryTimeout { .. }
                | DatabaseError::DatabaseUnavailable { .. }
        )
    }

    /// Check if the error is a connection issue
    pub fn is_connection_error(&self) -> bool {
        matches!(
            self,
            DatabaseError::ConnectionFailed(_)
                | DatabaseError::ConnectionTimeout { .. }
                | DatabaseError::DatabaseUnavailable { .. }
        )
    }
}
