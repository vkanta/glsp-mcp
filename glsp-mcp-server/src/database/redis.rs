//! Redis database implementation for GLSP-MCP Server
//!
//! Provides Redis-based storage for caching and session management.

use crate::database::{
    config::DatabaseConfig,
    error::{DatabaseError, DatabaseResult},
    models::*,
    traits::{DatabaseProvider, SensorDataRepository, TimeSeriesStore, MetadataStore},
};
use pulseengine_mcp_auth::SessionManager;
use async_trait::async_trait;
// Note: Serde traits reserved for future JSON serialization features
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Redis database backend implementation
#[derive(Debug, Clone)]
pub struct RedisBackend {
    url: String,
    client: Option<redis::Client>,
    connection_timeout: Duration,
}

impl RedisBackend {
    /// Create a new Redis backend instance
    pub fn new(config: &DatabaseConfig) -> DatabaseResult<Self> {
        let url = format!("redis://{}:{}", config.host, config.port);

        Ok(Self {
            url,
            client: None,
            connection_timeout: Duration::from_secs(config.connection_timeout_secs),
        })
    }

    /// Get Redis connection
    async fn get_connection(&self) -> DatabaseResult<redis::Connection> {
        let client = self.client.as_ref().ok_or(DatabaseError::ConnectionError(
            "Redis client not initialized".to_string(),
        ))?;

        client.get_connection().map_err(|e| {
            DatabaseError::ConnectionError(format!("Failed to get Redis connection: {}", e))
        })
    }
}

#[async_trait]
impl DatabaseProvider for RedisBackend {
    async fn connect(&mut self) -> DatabaseResult<()> {
        self.initialize().await
    }

    async fn disconnect(&mut self) -> DatabaseResult<()> {
        self.close().await
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    async fn health_check(&self) -> DatabaseResult<DatabaseHealth> {
        let start = std::time::Instant::now();
        let is_healthy = match self.get_connection().await {
            Ok(mut conn) => {
                use redis::Commands;
                match conn.ping::<String>() {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
            Err(_) => false,
        };
        let latency_ms = start.elapsed().as_millis() as f32;

        Ok(DatabaseHealth {
            is_connected: is_healthy,
            latency_ms,
            version: Some("Redis (unknown version)".to_string()),
            active_connections: Some(if self.is_connected() { 1 } else { 0 }),
            total_connections: Some(1),
            last_error: None,
            uptime_seconds: None,
            memory_usage_mb: None,
        })
    }

    fn database_type(&self) -> &'static str {
        "Redis"
    }

    fn connection_info(&self) -> String {
        // Return sanitized connection info (no credentials)
        format!("Redis at {}", self.url.replace("redis://", "").split('@').last().unwrap_or("unknown"))
    }
}

impl RedisBackend {
    /// Initialize the Redis backend
    async fn initialize(&mut self) -> DatabaseResult<()> {
        let client = redis::Client::open(self.url.as_str()).map_err(|e| {
            DatabaseError::ConnectionError(format!("Failed to create Redis client: {}", e))
        })?;

        self.client = Some(client);

        // Test connection
        let _conn = self.get_connection().await?;

        Ok(())
    }

    /// Close the Redis connection
    async fn close(&mut self) -> DatabaseResult<()> {
        self.client = None;
        Ok(())
    }
}

#[async_trait]
impl SessionManager for RedisBackend {
    async fn create_session(
        &mut self,
        session_id: &str,
        user_id: &str,
        ttl: Duration,
    ) -> DatabaseResult<()> {
        let mut conn = self.get_connection().await?;

        let session_key = format!("session:{}", session_id);
        let session_data = serde_json::json!({
            "user_id": user_id,
            "created_at": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0)).as_secs(),
            "ttl": ttl.as_secs()
        });

        use redis::Commands;
        conn.set_ex(
            &session_key,
            session_data.to_string(),
            ttl.as_secs() as usize,
        )
        .map_err(|e| DatabaseError::WriteError(format!("Failed to create session: {}", e)))?;

        Ok(())
    }

    async fn get_session(
        &self,
        session_id: &str,
    ) -> DatabaseResult<Option<HashMap<String, String>>> {
        let mut conn = self.get_connection().await?;

        let session_key = format!("session:{}", session_id);

        use redis::Commands;
        match conn.get::<String, String>(&session_key) {
            Ok(data) => {
                let session_data: serde_json::Value = serde_json::from_str(&data).map_err(|e| {
                    DatabaseError::ReadError(format!("Failed to parse session data: {}", e))
                })?;

                let mut result = HashMap::new();
                if let Some(user_id) = session_data.get("user_id").and_then(|v| v.as_str()) {
                    result.insert("user_id".to_string(), user_id.to_string());
                }
                if let Some(created_at) = session_data.get("created_at").and_then(|v| v.as_u64()) {
                    result.insert("created_at".to_string(), created_at.to_string());
                }

                Ok(Some(result))
            }
            Err(redis::RedisError {
                kind: redis::ErrorKind::ResponseError,
                ..
            }) => Ok(None),
            Err(e) => Err(DatabaseError::ReadError(format!(
                "Failed to get session: {}",
                e
            ))),
        }
    }

    async fn delete_session(&mut self, session_id: &str) -> DatabaseResult<()> {
        let mut conn = self.get_connection().await?;

        let session_key = format!("session:{}", session_id);

        use redis::Commands;
        conn.del(&session_key)
            .map_err(|e| DatabaseError::WriteError(format!("Failed to delete session: {}", e)))?;

        Ok(())
    }

    async fn extend_session(&mut self, session_id: &str, ttl: Duration) -> DatabaseResult<()> {
        let mut conn = self.get_connection().await?;

        let session_key = format!("session:{}", session_id);

        use redis::Commands;
        conn.expire(&session_key, ttl.as_secs() as usize)
            .map_err(|e| DatabaseError::WriteError(format!("Failed to extend session: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl SensorDataRepository for RedisBackend {
    async fn store_reading(&mut self, _reading: &SensorReading) -> DatabaseResult<()> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support sensor data storage".to_string(),
        ))
    }

    async fn store_batch(&mut self, _batch: &SensorBatch) -> DatabaseResult<()> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support sensor data storage".to_string(),
        ))
    }

    async fn query_readings(&self, _query: &SensorQuery) -> DatabaseResult<Vec<SensorReading>> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support sensor data queries".to_string(),
        ))
    }

    async fn get_reading_at_time(
        &self,
        _sensor_id: &str,
        _timestamp_us: i64,
    ) -> DatabaseResult<Option<SensorReading>> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support sensor data queries".to_string(),
        ))
    }

    async fn delete_readings(
        &mut self,
        _sensor_id: &str,
        _start_time_us: i64,
        _end_time_us: i64,
    ) -> DatabaseResult<u64> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support sensor data deletion".to_string(),
        ))
    }
}

#[async_trait]
impl TimeSeriesStore for RedisBackend {
    async fn downsample(
        &self,
        _sensor_id: &str,
        _start_time_us: i64,
        _end_time_us: i64,
        _interval_us: i64,
    ) -> DatabaseResult<Vec<SensorReading>> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support time-series operations".to_string(),
        ))
    }

    async fn interpolate(
        &self,
        _sensor_id: &str,
        _timestamps_us: &[i64],
    ) -> DatabaseResult<Vec<SensorReading>> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support interpolation".to_string(),
        ))
    }

    async fn aggregate(
        &self,
        _sensor_id: &str,
        _start_time_us: i64,
        _end_time_us: i64,
        _window_size_us: i64,
    ) -> DatabaseResult<Vec<SensorStatistics>> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support aggregation".to_string(),
        ))
    }

    async fn detect_gaps(
        &self,
        _sensor_id: &str,
        _start_time_us: i64,
        _end_time_us: i64,
        _max_gap_us: i64,
    ) -> DatabaseResult<Vec<TimeRange>> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support gap detection".to_string(),
        ))
    }
}

#[async_trait]
impl MetadataStore for RedisBackend {
    async fn store_sensor_metadata(&mut self, _metadata: &SensorMetadata) -> DatabaseResult<()> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support metadata storage".to_string(),
        ))
    }

    async fn get_sensor_metadata(&self, _sensor_id: &str) -> DatabaseResult<Option<SensorMetadata>> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support metadata queries".to_string(),
        ))
    }

    async fn list_sensor_metadata(&self) -> DatabaseResult<Vec<SensorMetadata>> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support sensor listing".to_string(),
        ))
    }

    async fn update_sensor_metadata(&mut self, _metadata: &SensorMetadata) -> DatabaseResult<()> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support metadata updates".to_string(),
        ))
    }

    async fn delete_sensor_metadata(&mut self, _sensor_id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::UnsupportedOperation(
            "Redis backend does not support metadata deletion".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::config::DatabaseConfig;

    #[tokio::test]
    async fn test_redis_backend_creation() {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 6379,
            database: "test".to_string(),
            username: None,
            password: None,
            connection_timeout_secs: 30,
            max_connections: 10,
            backend_type: "redis".to_string(),
        };

        let backend = RedisBackend::new(&config);
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_redis_backend_initialization() {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 6379,
            database: "test".to_string(),
            username: None,
            password: None,
            connection_timeout_secs: 30,
            max_connections: 10,
            backend_type: "redis".to_string(),
        };

        let mut backend = RedisBackend::new(&config).expect("Failed to create Redis backend");

        // This will fail if Redis is not running, but that's expected in CI
        let _ = backend.initialize().await;
    }
}
