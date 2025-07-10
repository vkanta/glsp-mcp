//! Database abstraction layer for GLSP-MCP Server
//!
//! Provides exchangeable database backends for sensor data, simulation state,
//! and other time-series data through a unified SDK interface.

pub mod config;
pub mod dataset;
pub mod error;
pub mod factory;
pub mod models;
pub mod traits;

#[cfg(test)]
pub mod tests;

#[cfg(test)]
pub mod factory_tests;

// Concrete implementations
#[cfg(feature = "postgresql")]
pub mod postgresql;

#[cfg(feature = "influxdb")]
pub mod influxdb;

#[cfg(feature = "redis")]
pub mod redis;

// Re-exports for convenience
pub use config::DatabaseConfig;
pub use dataset::*;
pub use error::{DatabaseError, DatabaseResult};
pub use factory::DatabaseFactory;
pub use models::*;
pub use traits::*;

/// Version of the database schema/API
pub const DATABASE_API_VERSION: &str = "1.0.0";

/// Default connection timeout in seconds
pub const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 30;

/// Default query timeout in seconds
pub const DEFAULT_QUERY_TIMEOUT_SECS: u64 = 60;
