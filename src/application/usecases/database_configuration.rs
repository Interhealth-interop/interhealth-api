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
            String::new() // company_id will be set from context
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
            company_id: Some(config.company_id),
            created_at: config.created_at.to_rfc3339(),
            updated_at: config.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_all_database_configurations(&self, page: i64, limit: i64) -> AppResult<PaginationResponse<DatabaseConfigurationEntity>> {
        let (configs, total) = self.repository.find_all(page, limit).await?;

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
            company_id: Some(updated.company_id),
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_database_configuration(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }

    pub async fn test_connection(&self, data: CreateDatabaseConfigurationDto) -> AppResult<serde_json::Value> {
        let connection_string = ConnectorFactory::build_connection_string(&data)?;

        let config = DatabaseConfig {
            connection_string,
            database_name: Some(data.database.clone()),
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
