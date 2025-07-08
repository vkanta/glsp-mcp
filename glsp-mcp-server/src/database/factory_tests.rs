//! Tests for database factory pattern and configuration-driven backend selection

#[cfg(test)]
mod tests {
    use crate::database::*;
    use crate::database::factory::DatabaseManager;
    use crate::backend::GlspConfig;
    use std::env;
    
    #[tokio::test]
    async fn test_database_factory_mock_backend() -> DatabaseResult<()> {
        let config = DatabaseConfig::mock();
        let backend = DatabaseFactory::create(config).await?;
        
        assert_eq!(backend.database_type(), "mock");
        assert!(backend.is_connected());
        
        let health = backend.health_check().await?;
        assert!(health.is_connected);
        assert_eq!(health.version, Some("mock-1.0.0".to_string()));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_database_factory_from_env() -> DatabaseResult<()> {
        // Set environment variables for mock backend
        env::set_var("GLSP_DB_BACKEND", "mock");
        env::set_var("GLSP_DB_HOST", "localhost");
        env::set_var("GLSP_DB_PORT", "5432");
        
        let backend = DatabaseFactory::from_env().await?;
        assert_eq!(backend.database_type(), "mock");
        
        // Clean up
        env::remove_var("GLSP_DB_BACKEND");
        env::remove_var("GLSP_DB_HOST");
        env::remove_var("GLSP_DB_PORT");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_database_manager_lifecycle() -> DatabaseResult<()> {
        let config = DatabaseConfig::mock();
        let manager = DatabaseManager::new(config).await?;
        
        // Initially healthy after creation
        assert!(manager.is_healthy().await);
        
        // Test reconnection
        manager.reconnect().await?;
        assert!(manager.is_healthy().await);
        
        // Test shutdown
        manager.shutdown().await?;
        
        Ok(())
    }
    
    #[test]
    fn test_glsp_config_database_conversion() {
        let mut config = GlspConfig::default();
        
        // Test disabled database
        config.enable_database = false;
        let db_config = config.to_database_config().unwrap();
        assert!(matches!(db_config.backend, config::DatabaseBackend::Mock));
        
        // Test PostgreSQL backend
        config.enable_database = true;
        config.database_backend = "postgresql".to_string();
        config.database_host = "localhost".to_string();
        config.database_port = 5432;
        config.database_name = "test_db".to_string();
        config.database_user = Some("test_user".to_string());
        
        let db_config = config.to_database_config().unwrap();
        assert!(matches!(db_config.backend, config::DatabaseBackend::PostgreSQL));
        assert_eq!(db_config.connection.host, "localhost");
        assert_eq!(db_config.connection.port, 5432);
        assert_eq!(db_config.connection.database, "test_db");
        assert_eq!(db_config.connection.username, Some("test_user".to_string()));
    }
    
    #[test]
    fn test_glsp_config_invalid_backend() {
        let mut config = GlspConfig::default();
        config.enable_database = true;
        config.database_backend = "invalid_backend".to_string();
        
        let result = config.to_database_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown database backend"));
    }
    
    #[tokio::test]
    async fn test_feature_conditional_compilation() {
        let mut config = DatabaseConfig::mock();
        config.backend = config::DatabaseBackend::PostgreSQL;
        
        let result = DatabaseFactory::create(config).await;
        
        #[cfg(feature = "postgresql")]
        {
            // PostgreSQL feature is enabled - may succeed or fail depending on environment
            // but should not fail with FeatureNotSupported
            match result {
                Ok(_) => {
                    // Successfully created (unlikely in test environment without DB)
                }
                Err(DatabaseError::FeatureNotSupported { .. }) => {
                    panic!("Should not fail with FeatureNotSupported when feature is enabled");
                }
                Err(_) => {
                    // Other errors are acceptable (connection failures, etc.)
                }
            }
        }
        
        #[cfg(not(feature = "postgresql"))]
        {
            // PostgreSQL feature is disabled, should fail with feature not supported
            assert!(result.is_err());
            if let Err(DatabaseError::FeatureNotSupported { feature }) = result {
                assert!(feature.contains("PostgreSQL"));
            } else {
                panic!("Expected FeatureNotSupported error");
            }
        }
    }
    
    #[tokio::test]
    async fn test_database_backend_string_conversion() {
        use config::DatabaseBackend;
        
        assert_eq!(DatabaseBackend::PostgreSQL.as_str(), "postgresql");
        assert_eq!(DatabaseBackend::InfluxDB.as_str(), "influxdb");
        assert_eq!(DatabaseBackend::Redis.as_str(), "redis");
        assert_eq!(DatabaseBackend::SQLite.as_str(), "sqlite");
        assert_eq!(DatabaseBackend::Mock.as_str(), "mock");
    }
    
    #[tokio::test]
    async fn test_database_configuration_validation() {
        // Test valid configuration
        let valid_config = DatabaseConfig::mock();
        assert!(valid_config.validate().is_ok());
        
        // Test configuration with invalid database name
        let mut invalid_config = DatabaseConfig::mock();
        invalid_config.backend = config::DatabaseBackend::PostgreSQL; // PostgreSQL requires database name
        invalid_config.connection.database = "".to_string(); // Empty database name
        
        assert!(invalid_config.validate().is_err());
    }
    
    #[tokio::test]
    async fn test_environment_variable_password_loading() {
        let mut config = GlspConfig::default();
        config.enable_database = true;
        config.database_backend = "mock".to_string();
        
        // Test with password in environment
        env::set_var("GLSP_DB_PASSWORD", "secret_password");
        
        let db_config = config.to_database_config().unwrap();
        assert_eq!(db_config.connection.password, Some("secret_password".to_string()));
        
        // Clean up
        env::remove_var("GLSP_DB_PASSWORD");
        
        // Test without password in environment
        let db_config = config.to_database_config().unwrap();
        assert_eq!(db_config.connection.password, None);
    }
    
    #[tokio::test]
    async fn test_supported_backend_types() {
        let backends = vec![
            ("postgresql", config::DatabaseBackend::PostgreSQL),
            ("postgres", config::DatabaseBackend::PostgreSQL),
            ("influxdb", config::DatabaseBackend::InfluxDB),
            ("influx", config::DatabaseBackend::InfluxDB),
            ("redis", config::DatabaseBackend::Redis),
            ("sqlite", config::DatabaseBackend::SQLite),
            ("mock", config::DatabaseBackend::Mock),
        ];
        
        for (backend_name, expected_backend) in backends {
            let mut config = GlspConfig::default();
            config.enable_database = true;
            config.database_backend = backend_name.to_string();
            
            let db_config = config.to_database_config().unwrap();
            assert_eq!(db_config.backend, expected_backend);
        }
    }
}