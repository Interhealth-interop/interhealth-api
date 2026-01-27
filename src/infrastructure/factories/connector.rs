use crate::utils::AppError;
use crate::infrastructure::adapters::{MongoDBConnector, MongoDBConfig, OracleConnector};
use crate::domain::dtos::CreateDatabaseConfigurationDto;

/// Enum representing different database types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseType {
    MongoDB,
    PostgreSQL,
    Oracle,
    MySQL,
}

/// Enum representing different client aliases
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientAlias {
    MV,
    TASY,
    TOTVS,
    Custom(String),
}

impl ClientAlias {
    pub fn from_str(alias: &str) -> Self {
        match alias.to_uppercase().as_str() {
            "MV" => ClientAlias::MV,
            "TASY" => ClientAlias::TASY,
            "TOTVS" => ClientAlias::TOTVS,
            _ => ClientAlias::Custom(alias.to_string()),
        }
    }
}

/// Configuration for database connection
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub database_name: Option<String>,
    pub max_connections: Option<u32>,
}

/// Factory for creating database connections
pub struct ConnectorFactory;

impl ConnectorFactory {
    /// Determine database type based on client alias
    pub fn get_database_type(alias: &ClientAlias) -> DatabaseType {
        match alias {
            ClientAlias::MV => DatabaseType::Oracle,
            ClientAlias::TASY => DatabaseType::Oracle,
            ClientAlias::TOTVS => DatabaseType::Oracle,
            ClientAlias::Custom(_) => DatabaseType::MongoDB, // Default to MongoDB
        }
    }

    /// Build connection string based on database type and connection parameters
    pub fn build_connection_string(data: &CreateDatabaseConfigurationDto) -> Result<String, AppError> {
        let connection_string = match data.db_type.to_uppercase().as_str() {
            "MV" | "TASY" | "TOTVS" => {
                format!(
                    "oracle://{}:{}@{}:{}/{}",
                    data.username, data.password, data.host, data.port, data.database
                )
            }
            "MONGODB" => {
                format!(
                    "mongodb://{}:{}@{}:{}/{}",
                    data.username, data.password, data.host, data.port, data.database
                )
            }
            "POSTGRESQL" => {
                format!(
                    "postgresql://{}:{}@{}:{}/{}",
                    data.username, data.password, data.host, data.port, data.database
                )
            }
            "MYSQL" => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    data.username, data.password, data.host, data.port, data.database
                )
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Unsupported database type: {}",
                    data.db_type
                )));
            }
        };
        Ok(connection_string)
    }

    /// Create a database connection based on alias
    pub async fn create_connection(
        alias: &str,
        config: DatabaseConfig,
    ) -> Result<Box<dyn DatabaseConnection>, AppError> {
        let client_alias = ClientAlias::from_str(alias);
        let db_type = Self::get_database_type(&client_alias);

        match db_type {
            DatabaseType::MongoDB => {
                let connection = MongoDBConnection::new(config).await?;
                Ok(Box::new(connection))
            }
            DatabaseType::Oracle => {
                let connection = OracleConnection::new(config).await?;
                Ok(Box::new(connection))
            }
            DatabaseType::PostgreSQL => {
                let connection = PostgreSQLConnection::new(config).await?;
                Ok(Box::new(connection))
            }
            DatabaseType::MySQL => {
                let connection = MySQLConnection::new(config).await?;
                Ok(Box::new(connection))
            }
        }
    }

    /// Create connection by database type directly
    pub async fn create_by_type(
        db_type: DatabaseType,
        config: DatabaseConfig,
    ) -> Result<Box<dyn DatabaseConnection>, AppError> {
        match db_type {
            DatabaseType::MongoDB => {
                let connection = MongoDBConnection::new(config).await?;
                Ok(Box::new(connection))
            }
            DatabaseType::Oracle => {
                let connection = OracleConnection::new(config).await?;
                Ok(Box::new(connection))
            }
            DatabaseType::PostgreSQL => {
                let connection = PostgreSQLConnection::new(config).await?;
                Ok(Box::new(connection))
            }
            DatabaseType::MySQL => {
                let connection = MySQLConnection::new(config).await?;
                Ok(Box::new(connection))
            }
        }
    }
}

/// Trait for database connections
pub trait DatabaseConnection: Send + Sync {
    fn get_type(&self) -> DatabaseType;
    fn is_connected(&self) -> bool;
    fn get_connection_string(&self) -> &str;
}

/// MongoDB connection implementation
struct MongoDBConnection {
    connector: MongoDBConnector,
    config: DatabaseConfig,
}

impl MongoDBConnection {
    async fn new(config: DatabaseConfig) -> Result<Self, AppError> {
        let mongo_config = MongoDBConfig::new(config.connection_string.clone())
            .with_database_name(config.database_name.clone().unwrap_or_else(|| "interhealth".to_string()));
        
        let connector = MongoDBConnector::from_config(mongo_config).await?;
        
        Ok(Self {
            connector,
            config,
        })
    }
    
    pub fn connector(&self) -> &MongoDBConnector {
        &self.connector
    }
}

impl DatabaseConnection for MongoDBConnection {
    fn get_type(&self) -> DatabaseType {
        DatabaseType::MongoDB
    }

    fn is_connected(&self) -> bool {
        // MongoDB connector is always "connected" once created successfully
        true
    }

    fn get_connection_string(&self) -> &str {
        &self.config.connection_string
    }
}

/// Oracle connection implementation
struct OracleConnection {
    connector: OracleConnector,
    config: DatabaseConfig,
}

impl OracleConnection {
    async fn new(config: DatabaseConfig) -> Result<Self, AppError> {
        let connector = OracleConnector::new(&config.connection_string).await?;
        
        Ok(Self {
            connector,
            config,
        })
    }
    
    pub fn connector(&self) -> &OracleConnector {
        &self.connector
    }
}

impl DatabaseConnection for OracleConnection {
    fn get_type(&self) -> DatabaseType {
        DatabaseType::Oracle
    }

    fn is_connected(&self) -> bool {
        self.connector.is_connected()
    }

    fn get_connection_string(&self) -> &str {
        &self.config.connection_string
    }
}

/// PostgreSQL connection implementation
struct PostgreSQLConnection {
    config: DatabaseConfig,
    connected: bool,
}

impl PostgreSQLConnection {
    async fn new(_config: DatabaseConfig) -> Result<Self, AppError> {
        Err(AppError::BadRequest(
            "PostgreSQL connections are not yet implemented. Please use MongoDB.".to_string()
        ))
    }
}

impl DatabaseConnection for PostgreSQLConnection {
    fn get_type(&self) -> DatabaseType {
        DatabaseType::PostgreSQL
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_connection_string(&self) -> &str {
        &self.config.connection_string
    }
}

/// MySQL connection implementation
struct MySQLConnection {
    config: DatabaseConfig,
    connected: bool,
}

impl MySQLConnection {
    async fn new(_config: DatabaseConfig) -> Result<Self, AppError> {
        Err(AppError::BadRequest(
            "MySQL connections are not yet implemented. Please use MongoDB.".to_string()
        ))
    }
}

impl DatabaseConnection for MySQLConnection {
    fn get_type(&self) -> DatabaseType {
        DatabaseType::MySQL
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_connection_string(&self) -> &str {
        &self.config.connection_string
    }
}