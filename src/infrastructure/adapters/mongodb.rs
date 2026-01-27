use mongodb::{Client, Database};
use crate::utils::AppError;

/// MongoDB database connection configuration
#[derive(Debug, Clone)]
pub struct MongoDBConfig {
    pub connection_string: String,
    pub database_name: Option<String>,
    pub max_pool_size: Option<u32>,
    pub min_pool_size: Option<u32>,
}

impl MongoDBConfig {
    /// Create a new MongoDB configuration
    pub fn new(connection_string: String) -> Self {
        Self {
            connection_string,
            database_name: None,
            max_pool_size: None,
            min_pool_size: None,
        }
    }

    /// Set database name
    pub fn with_database_name(mut self, database_name: String) -> Self {
        self.database_name = Some(database_name);
        self
    }

    /// Set max pool size
    pub fn with_max_pool_size(mut self, size: u32) -> Self {
        self.max_pool_size = Some(size);
        self
    }

    /// Set min pool size
    pub fn with_min_pool_size(mut self, size: u32) -> Self {
        self.min_pool_size = Some(size);
        self
    }

    /// Extract database name from connection string
    fn extract_db_name(&self) -> Option<String> {
        // Format: mongodb+srv://user:pass@host/dbname?options
        if let Some(after_slash) = self.connection_string.split('/').nth(3) {
            if let Some(db_name) = after_slash.split('?').next() {
                if !db_name.is_empty() {
                    return Some(db_name.to_string());
                }
            }
        }
        None
    }

    /// Get the database name (from config or connection string)
    pub fn get_database_name(&self) -> String {
        self.database_name
            .clone()
            .or_else(|| self.extract_db_name())
            .unwrap_or_else(|| "interhealth".to_string())
    }
}

/// MongoDB database connector
pub struct MongoDBConnector {
    client: Client,
    database: Database,
    config: MongoDBConfig,
}

impl MongoDBConnector {
    /// Create a new MongoDB connector
    pub async fn new(connection_string: &str) -> Result<Self, AppError> {
        let config = MongoDBConfig::new(connection_string.to_string());
        Self::from_config(config).await
    }

    /// Create connector from config
    pub async fn from_config(config: MongoDBConfig) -> Result<Self, AppError> {
        let client = Client::with_uri_str(&config.connection_string)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to connect to MongoDB: {}", e)))?;

        let db_name = config.get_database_name();
        let database = client.database(&db_name);

        Ok(Self {
            client,
            database,
            config,
        })
    }

    /// Get the database instance
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Get the client instance
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Test the connection
    pub async fn test_connection(&self) -> Result<bool, AppError> {
        self.database
            .run_command(mongodb::bson::doc! { "ping": 1 }, None)
            .await
            .map(|_| true)
            .map_err(|e| AppError::DatabaseError(format!("Falha ao realizar a conexão: {}", e)))
    }

    /// Get connection configuration
    pub fn get_config(&self) -> &MongoDBConfig {
        &self.config
    }

    /// Get database name
    pub fn database_name(&self) -> &str {
        self.database.name()
    }

    /// List all collections in the database
    pub async fn list_collections(&self) -> Result<Vec<String>, AppError> {
        self.database
            .list_collection_names(None)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to list collections: {}", e)))
    }
}

/// Helper function to connect to MongoDB and return a Database instance
/// This maintains backward compatibility with the existing codebase
pub async fn connect(mongo_url: &str) -> Result<Database, AppError> {
    let connector = MongoDBConnector::new(mongo_url).await?;
    Ok(connector.database().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_db_name() {
        let config = MongoDBConfig::new(
            "mongodb+srv://user:pass@cluster.mongodb.net/mydb?retryWrites=true".to_string()
        );
        assert_eq!(config.get_database_name(), "mydb");
    }

    #[test]
    fn test_default_db_name() {
        let config = MongoDBConfig::new("mongodb://localhost:27017".to_string());
        assert_eq!(config.get_database_name(), "interhealth");
    }

    #[test]
    fn test_config_builder() {
        let config = MongoDBConfig::new("mongodb://localhost:27017".to_string())
            .with_database_name("testdb".to_string())
            .with_max_pool_size(10)
            .with_min_pool_size(2);

        assert_eq!(config.get_database_name(), "testdb");
        assert_eq!(config.max_pool_size, Some(10));
        assert_eq!(config.min_pool_size, Some(2));
    }
}
