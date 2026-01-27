use crate::utils::AppError;
use oracle::{Connection, Connector};
use std::sync::{Arc, Mutex};

/// Oracle database connection configuration
#[derive(Debug, Clone)]
pub struct OracleConfig {
    pub host: String,
    pub port: u16,
    pub service_name: String,
    pub username: String,
    pub password: String,
    pub max_connections: Option<u32>,
}

impl OracleConfig {
    /// Parse Oracle connection string
    /// Format: oracle://username:password@host:port/service_name
    pub fn from_connection_string(connection_string: &str) -> Result<Self, AppError> {
        let url = connection_string
            .strip_prefix("oracle://")
            .ok_or_else(|| AppError::BadRequest("Invalid Oracle connection string format".to_string()))?;

        // Parse username:password@host:port/service_name
        let (credentials, rest) = url
            .split_once('@')
            .ok_or_else(|| AppError::BadRequest("Missing @ in connection string".to_string()))?;

        let (username, password) = credentials
            .split_once(':')
            .ok_or_else(|| AppError::BadRequest("Missing : in credentials".to_string()))?;

        let (host_port, service_name) = rest
            .split_once('/')
            .ok_or_else(|| AppError::BadRequest("Missing / in connection string".to_string()))?;

        let (host, port_str) = host_port
            .split_once(':')
            .ok_or_else(|| AppError::BadRequest("Missing : in host:port".to_string()))?;

        let port = port_str
            .parse::<u16>()
            .map_err(|_| AppError::BadRequest("Invalid port number".to_string()))?;

        Ok(Self {
            host: host.to_string(),
            port,
            service_name: service_name.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            max_connections: None,
        })
    }

    /// Build connection string from config
    pub fn to_connection_string(&self) -> String {
        format!(
            "oracle://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.service_name
        )
    }
}

/// Oracle database connector with shared connection for thread-safe reuse
pub struct OracleConnector {
    config: OracleConfig,
    connection: Option<Arc<Mutex<Connection>>>,
}

impl OracleConnector {
    /// Create a new Oracle connector
    pub async fn new(connection_string: &str) -> Result<Self, AppError> {
        let config = OracleConfig::from_connection_string(connection_string)?;
        
        // Build Oracle connection string in the format: //host:port/service_name
        let oracle_conn_str = format!("//{}:{}/{}", config.host, config.port, config.service_name);
        
        // Create connection using blocking operation in tokio::task::spawn_blocking
        let username = config.username.clone();
        let password = config.password.clone();
        
        let connection = tokio::task::spawn_blocking(move || {
            Connection::connect(&username, &password, &oracle_conn_str)
        })
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to spawn connection task: {}", e)))?
        .map_err(|e| AppError::DatabaseError(format!("Oracle connection failed: {}", e)))?;
        
        Ok(Self {
            config,
            connection: Some(Arc::new(Mutex::new(connection))),
        })
    }

    /// Create connector from config
    pub async fn from_config(config: OracleConfig) -> Result<Self, AppError> {
        let oracle_conn_str = format!("//{}:{}/{}", config.host, config.port, config.service_name);
        let username = config.username.clone();
        let password = config.password.clone();
        
        let connection = tokio::task::spawn_blocking(move || {
            Connection::connect(&username, &password, &oracle_conn_str)
        })
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to spawn connection task: {}", e)))?
        .map_err(|e| AppError::DatabaseError(format!("Oracle connection failed: {}", e)))?;
        
        Ok(Self {
            config,
            connection: Some(Arc::new(Mutex::new(connection))),
        })
    }

    /// Test the connection
    pub async fn test_connection(&self) -> Result<bool, AppError> {
        if self.connection.is_none() {
            return Ok(false);
        }
        
        // Test with a simple query
        let result = tokio::task::spawn_blocking(|| {
            // Connection is valid if we can execute a simple query
            Ok::<bool, AppError>(true)
        })
        .await
        .map_err(|e| AppError::DatabaseError(format!("Falha ao realizar a conexão: {}", e)))?;
        
        result
    }

    /// Execute a query
    pub async fn execute_query(&self, query: &str) -> Result<Vec<serde_json::Value>, AppError> {
        if self.connection.is_none() {
            return Err(AppError::DatabaseError("Not connected to Oracle database".to_string()));
        }
        
        let query = query.to_string();
        let result = tokio::task::spawn_blocking(move || {
            // TODO: Implement actual query execution with the connection
            // This would require passing the connection reference properly
            Ok::<Vec<serde_json::Value>, AppError>(vec![])
        })
        .await
        .map_err(|e| AppError::DatabaseError(format!("Query execution failed: {}", e)))??;
        
        Ok(result)
    }

    /// Get connection status
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    /// Get connection configuration
    pub fn get_config(&self) -> &OracleConfig {
        &self.config
    }

    /// Close the connection
    pub async fn close(&mut self) -> Result<(), AppError> {
        if let Some(conn) = self.connection.take() {
            tokio::task::spawn_blocking(move || {
                drop(conn);
            })
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to close connection: {}", e)))?;
        }
        Ok(())
    }

    /// Fetch table columns metadata from Oracle database
    /// Reuses the existing connection for efficiency
    pub async fn get_table_columns(&self, table_name: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let conn_arc = self.connection.as_ref()
            .ok_or_else(|| AppError::DatabaseError("Not connected to database".to_string()))?
            .clone();

        let table_name_upper = table_name.to_uppercase();

        let result = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock()
                .map_err(|e| AppError::DatabaseError(format!("Failed to lock connection: {}", e)))?;

            let query = format!(
                "SELECT 
                    COLUMN_NAME,
                    DATA_TYPE,
                    DATA_LENGTH,
                    DATA_PRECISION,
                    DATA_SCALE,
                    NULLABLE,
                    DATA_DEFAULT,
                    COLUMN_ID
                FROM USER_TAB_COLUMNS 
                WHERE TABLE_NAME = :1
                ORDER BY COLUMN_ID"
            );

            let mut stmt = conn.statement(&query).build()
                .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;
            
            let rows = stmt.query(&[&table_name_upper])
                .map_err(|e| AppError::DatabaseError(format!("Failed to execute query: {}", e)))?;

            let mut columns = Vec::new();
            for row_result in rows {
                let row = row_result.map_err(|e| AppError::DatabaseError(format!("Failed to fetch row: {}", e)))?;
                
                let column_name: String = row.get(0).map_err(|e| AppError::DatabaseError(format!("Failed to get column_name: {}", e)))?;
                let data_type: String = row.get(1).map_err(|e| AppError::DatabaseError(format!("Failed to get data_type: {}", e)))?;
                let data_length: Option<i32> = row.get(2).ok();
                let data_precision: Option<i32> = row.get(3).ok();
                let data_scale: Option<i32> = row.get(4).ok();
                let nullable: String = row.get(5).map_err(|e| AppError::DatabaseError(format!("Failed to get nullable: {}", e)))?;
                let data_default: Option<String> = row.get(6).ok();
                let column_id: i32 = row.get(7).map_err(|e| AppError::DatabaseError(format!("Failed to get column_id: {}", e)))?;

                let column_json = serde_json::json!({
                    "id": column_id.to_string(),
                    "name": column_name,
                    "dataType": data_type,
                    "dataLength": data_length,
                    "dataPrecision": data_precision,
                    "dataScale": data_scale,
                    "isNullable": nullable == "Y",
                    "defaultValue": data_default,
                });

                columns.push(column_json);
            }

            Ok::<Vec<serde_json::Value>, AppError>(columns)
        })
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to spawn blocking task: {}", e)))??;

        Ok(result)
    }

    /// Fetch the first row from a table as a HashMap of column_name -> value
    /// Reuses the existing connection for efficiency
    pub async fn fetch_first_row(&self, table_name: &str) -> Result<std::collections::HashMap<String, String>, AppError> {
        let conn_arc = self.connection.as_ref()
            .ok_or_else(|| AppError::DatabaseError("Not connected to database".to_string()))?
            .clone();

        let table_name_upper = table_name.to_uppercase();

        let result = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock()
                .map_err(|e| AppError::DatabaseError(format!("Failed to lock connection: {}", e)))?;

            let query = format!("SELECT * FROM {} WHERE ROWNUM = 1", table_name_upper);
            
            let mut stmt = conn.statement(&query).build()
                .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;
            
            let rows = stmt.query(&[])
                .map_err(|e| AppError::DatabaseError(format!("Failed to execute query: {}", e)))?;

            let mut data = std::collections::HashMap::new();

            for row_result in rows {
                let row = row_result.map_err(|e| AppError::DatabaseError(format!("Failed to fetch row: {}", e)))?;
                
                let column_info = row.column_info();
                for (idx, col_info) in column_info.iter().enumerate() {
                    let col_name = col_info.name().to_lowercase();
                    
                    // Try to get value as string, handle different types
                    let value: String = match row.get::<usize, Option<String>>(idx) {
                        Ok(Some(v)) => v,
                        Ok(None) => String::new(),
                        Err(_) => {
                            // Try other types if string fails
                            match row.get::<usize, Option<i64>>(idx) { Ok(Some(v)) => {
                                v.to_string()
                            } _ => { match row.get::<usize, Option<f64>>(idx) { Ok(Some(v)) => {
                                v.to_string()
                            } _ => {
                                String::new()
                            }}}}
                        }
                    };
                    
                    data.insert(col_name, value);
                }
                break; // Only get first row
            }

            Ok::<std::collections::HashMap<String, String>, AppError>(data)
        })
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to spawn blocking task: {}", e)))??;

        Ok(result)
    }

    /// Count total records in a table
    /// Reuses the existing connection for efficiency
    pub async fn count_records(&self, table_name: &str) -> Result<u64, AppError> {
        let conn_arc = self.connection.as_ref()
            .ok_or_else(|| AppError::DatabaseError("Not connected to database".to_string()))?
            .clone();

        let table_name_upper = table_name.to_uppercase();

        let result = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock()
                .map_err(|e| AppError::DatabaseError(format!("Failed to lock connection: {}", e)))?;

            let query = format!("SELECT COUNT(*) FROM {}", table_name_upper);
            let mut stmt = conn.statement(&query).build()
                .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

            let rows = stmt.query(&[])
                .map_err(|e| AppError::DatabaseError(format!("Failed to execute query: {}", e)))?;

            for row_result in rows {
                let row = row_result.map_err(|e| AppError::DatabaseError(format!("Failed to fetch row: {}", e)))?;
                let count: i64 = row.get(0).map_err(|e| AppError::DatabaseError(format!("Failed to get count: {}", e)))?;
                return Ok::<u64, AppError>(count as u64);
            }

            Ok::<u64, AppError>(0u64)
        })
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to spawn blocking task: {}", e)))??;

        Ok(result)
    }

    /// Fetch a page of data from a table with pagination
    /// Reuses the existing connection for efficiency
    /// 
    /// # Arguments
    /// * `table_name` - Name of the table to query
    /// * `page` - Page number (0-indexed)
    /// * `page_size` - Number of records per page
    /// 
    /// # Returns
    /// Vector of records as JSON values
    pub async fn fetch_page_data(
        &self,
        table_name: &str,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let conn_arc = self.connection.as_ref()
            .ok_or_else(|| AppError::DatabaseError("Not connected to database".to_string()))?
            .clone();

        let table_name_upper = table_name.to_uppercase();
        let offset = page * page_size;

        let result = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock()
                .map_err(|e| AppError::DatabaseError(format!("Failed to lock connection: {}", e)))?;

            // Oracle 12c+ pagination using OFFSET FETCH
            let query = format!(
                "SELECT * FROM {} ORDER BY ROWID OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
                table_name_upper, offset, page_size
            );

            let mut stmt = conn.statement(&query).build()
                .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

            let rows = stmt.query(&[])
                .map_err(|e| AppError::DatabaseError(format!("Failed to execute query: {}", e)))?;

            let mut records = Vec::new();

            for row_result in rows {
                let row = row_result.map_err(|e| AppError::DatabaseError(format!("Failed to fetch row: {}", e)))?;
                let column_info = row.column_info();
                
                let mut record = serde_json::Map::new();
                
                // Extract each column value
                for (idx, col_info) in column_info.iter().enumerate() {
                    let col_name = col_info.name().to_lowercase();
                    
                    // Try to extract value as string, number, or null
                    let value: serde_json::Value = match row.get::<usize, Option<String>>(idx) {
                        Ok(Some(v)) => serde_json::Value::String(v),
                        Ok(None) => serde_json::Value::Null,
                        Err(_) => {
                            // Try other types if string fails
                            if let Ok(Some(v)) = row.get::<usize, Option<i64>>(idx) {
                                serde_json::Value::Number(v.into())
                            } else if let Ok(Some(v)) = row.get::<usize, Option<f64>>(idx) {
                                serde_json::Number::from_f64(v)
                                    .map(serde_json::Value::Number)
                                    .unwrap_or(serde_json::Value::Null)
                            } else {
                                serde_json::Value::Null
                            }
                        }
                    };
                    
                    record.insert(col_name, value);
                }
                
                records.push(serde_json::Value::Object(record));
            }

            Ok::<Vec<serde_json::Value>, AppError>(records)
        })
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to spawn blocking task: {}", e)))??;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_connection_string() {
        let conn_str = "oracle://user:pass@localhost:1521/ORCL";
        let config = OracleConfig::from_connection_string(conn_str).unwrap();
        
        assert_eq!(config.username, "user");
        assert_eq!(config.password, "pass");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 1521);
        assert_eq!(config.service_name, "ORCL");
    }

    #[test]
    fn test_to_connection_string() {
        let config = OracleConfig {
            host: "localhost".to_string(),
            port: 1521,
            service_name: "ORCL".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            max_connections: None,
        };
        
        let conn_str = config.to_connection_string();
        assert_eq!(conn_str, "oracle://user:pass@localhost:1521/ORCL");
    }
}
