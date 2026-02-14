use std::sync::Arc;

use crate::domain::dtos::{CreateDatabaseConfigurationDto, UpdateDatabaseConfigurationDto, DatabaseConfigurationEntity};
use crate::infrastructure::repositories::DatabaseConfigurationRepository;
use crate::infrastructure::factories::{ConnectorFactory, DatabaseConfig};
use crate::utils::{AppError, AppResult, PaginationResponse};

pub struct DatabaseConfigurationUseCase {
    repository: Arc<DatabaseConfigurationRepository>,
}

impl DatabaseConfigurationUseCase {
    pub fn new(repository: Arc<DatabaseConfigurationRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_database_configuration(&self, data: CreateDatabaseConfigurationDto) -> AppResult<DatabaseConfigurationEntity> {
        let config = self.repository.create(
            data.name,
            data.db_type,
            data.version,
            data.host,
            data.port,
            data.database,
            data.username,
            data.password,
            data.auth_type,
            data.credentials,
            data.company_id.unwrap_or_default()
        ).await?;

        Ok(DatabaseConfigurationEntity {
            id: config.id.unwrap().to_hex(),
            name: config.name,
            db_type: config.db_type,
            version: config.version,
            host: config.host,
            port: config.port,
            database: config.database,
            username: config.username,
            password: config.password,
            auth_type: config.auth_type,
            credentials: config.credentials,
            company_id: Some(config.company_id),
            created_at: config.created_at.to_rfc3339(),
            updated_at: config.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_all_database_configurations(&self, page: i64, limit: i64, order_field: Option<String>, order_by: Option<String>) -> AppResult<PaginationResponse<DatabaseConfigurationEntity>> {
        use crate::utils::sort_helper::build_sort_document;
        
        let sort_document = build_sort_document(order_field, order_by);
        let (configs, total) = self.repository.find_all(page, limit, sort_document).await?;

        let entities: Vec<DatabaseConfigurationEntity> = configs.into_iter().map(|config| {
            DatabaseConfigurationEntity {
                id: config.id.unwrap().to_hex(),
                name: config.name,
                db_type: config.db_type,
                version: config.version,
                host: config.host,
                port: config.port,
                database: config.database,
                username: config.username,
                password: config.password,
                auth_type: config.auth_type,
                credentials: config.credentials,
                company_id: Some(config.company_id),
                created_at: config.created_at.to_rfc3339(),
                updated_at: config.updated_at.to_rfc3339(),
            }
        }).collect();

        Ok(PaginationResponse::new(
            "Configurações de banco de dados recuperadas com sucesso",
            entities,
            total,
            page,
            limit,
        ))
    }

    pub async fn get_database_configuration_by_id(&self, id: &str) -> AppResult<DatabaseConfigurationEntity> {
        let config = self.repository.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Database configuration not found".to_string()))?;

        Ok(DatabaseConfigurationEntity {
            id: config.id.unwrap().to_hex(),
            name: config.name,
            db_type: config.db_type,
            version: config.version,
            host: config.host,
            port: config.port,
            database: config.database,
            username: config.username,
            password: config.password,
            auth_type: config.auth_type,
            credentials: config.credentials,
            company_id: Some(config.company_id),
            created_at: config.created_at.to_rfc3339(),
            updated_at: config.updated_at.to_rfc3339(),
        })
    }


    pub async fn update_database_configuration(&self, id: &str, _data: UpdateDatabaseConfigurationDto) -> AppResult<DatabaseConfigurationEntity> {
        let updated = self.repository.update(
            id,
            _data.name,
            _data.db_type,
            _data.version,
            _data.host,
            _data.port,
            _data.database,
            _data.username,
            _data.password,
            _data.auth_type,
            _data.credentials,
            _data.company_id
        ).await?;

        Ok(DatabaseConfigurationEntity {
            id: updated.id.unwrap().to_hex(),
            name: updated.name,
            db_type: updated.db_type,
            version: updated.version,
            host: updated.host,
            port: updated.port,
            database: updated.database,
            username: updated.username,
            password: updated.password,
            auth_type: updated.auth_type,
            credentials: updated.credentials,
            company_id: Some(updated.company_id),
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_database_configuration(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }

    pub async fn test_connection(&self, data: CreateDatabaseConfigurationDto) -> AppResult<serde_json::Value> {
        // Check if this is an API connection
        if data.db_type.to_uppercase() == "API" {
            // Test API connection
            use crate::infrastructure::adapters::ApiConnector;
            
            match ApiConnector::new(&data.host, data.auth_type.clone(), data.credentials.clone()).await {
                Ok(connector) => {
                    match connector.test_connection().await {
                        Ok(is_connected) => {
                            Ok(serde_json::json!({
                                "success": true,
                                "message": "Conexão API estabelecida com sucesso",
                                "connected": is_connected,
                                "type": "API",
                                "host": data.host,
                                "authType": data.auth_type
                            }))
                        }
                        Err(e) => {
                            Ok(serde_json::json!({
                                "success": false,
                                "message": format!("Falha ao testar conexão API: {}", e),
                                "type": "API",
                                "host": data.host,
                                "authType": data.auth_type
                            }))
                        }
                    }
                }
                Err(e) => {
                    Ok(serde_json::json!({
                        "success": false,
                        "message": format!("Falha ao criar conector API: {}", e),
                        "type": "API",
                        "host": data.host,
                        "authType": data.auth_type
                    }))
                }
            }
        } else {
            // Test database connection
            let connection_string = ConnectorFactory::build_connection_string(&data)?;

            let config = DatabaseConfig {
                connection_string,
                database_name: data.database.clone(),
                max_connections: Some(10),
            };

            match ConnectorFactory::create_connection(&data.db_type, config).await {
                Ok(connection) => {
                    let is_connected = connection.is_connected();
                    let db_type = format!("{:?}", connection.get_type());
                    let host = data.host.clone();
                    let port = data.port;
                    let database = data.database.clone();
                    
                    drop(connection);
                    
                    Ok(serde_json::json!({
                        "success": true,
                        "message": "Conexão estabelecida com sucesso",
                        "connected": is_connected,
                        "type": db_type,
                        "host": host,
                        "port": port,
                        "database": database
                    }))
                }
                Err(e) => {
                    Ok(serde_json::json!({
                        "success": false,
                        "message": format!("Falha ao realizar a conexão: {}", e),
                        "type": data.db_type,
                        "host": data.host,
                        "port": data.port,
                        "database": data.database
                    }))
                }
            }
        }
    }
}
