use std::sync::Arc;

use crate::domain::dtos::{CreateDatabaseTableDto, UpdateDatabaseTableDto, DatabaseTableEntity, DatabaseColumnEntity, DatabaseTableReference};
use crate::infrastructure::repositories::{DatabaseTableRepository, DatabaseColumnRepository, DatabaseConfigurationRepository};
use crate::infrastructure::adapters::OracleConnector;
use crate::utils::{AppError, AppResult, PaginationResponse};

pub struct DatabaseTableUseCase {
    repository: Arc<DatabaseTableRepository>,
    column_repository: Arc<DatabaseColumnRepository>,
    config_repository: Option<Arc<DatabaseConfigurationRepository>>,
}

impl DatabaseTableUseCase {
    pub fn new(repository: Arc<DatabaseTableRepository>, column_repository: Arc<DatabaseColumnRepository>) -> Self {
        Self { repository, column_repository, config_repository: None }
    }

    pub fn with_config_repository(mut self, config_repository: Arc<DatabaseConfigurationRepository>) -> Self {
        self.config_repository = Some(config_repository);
        self
    }

    pub async fn create_database_table(&self, data: CreateDatabaseTableDto, company_id: String) -> AppResult<DatabaseTableEntity> {
        let table = self.repository.create(
            data.name.clone(),
            data.description.clone(),
            data.table_reference.clone(),
            data.table_type.clone(),
            data.entity_type.clone(),
            data.resource.clone(),
            company_id.clone(),
        ).await?;

        Ok(DatabaseTableEntity {
            id: table.id.unwrap().to_hex(),
            name: table.name,
            description: table.description,
            table_reference: table.table_reference,
            table_type: table.table_type,
            entity_type: table.entity_type,
            resource: table.resource,
            company_id: Some(company_id),
            created_at: table.created_at.to_rfc3339(),
            updated_at: table.updated_at.to_rfc3339(),
            columns: None,
        })
    }

    pub async fn get_all_database_tables(
        &self,
        page: i64,
        limit: i64,
        include_columns: bool,
        table_types: Option<Vec<String>>,
        entity_types: Option<Vec<String>>,
        table_references: Option<Vec<String>>,
    ) -> AppResult<PaginationResponse<DatabaseTableEntity>> {
        let (tables, total, _pages) = self.repository.find_all(
            page,
            limit,
            include_columns,
            table_types,
            entity_types,
            table_references,
        ).await?;

        let mut entities: Vec<DatabaseTableEntity> = Vec::new();
        
        for table in tables {
            let table_id = table.id.unwrap().to_hex();
            
            let columns = if include_columns {
                let cols = self.column_repository.find_by_table_id(&table_id).await?;
                Some(cols.into_iter().map(|col| {
                    DatabaseColumnEntity {
                        id: col.id.unwrap().to_hex(),
                        name: col.name,
                        reference: col.reference,
                        data_type: col.data_type,
                        is_nullable: col.is_nullable,
                        is_primary_key: col.is_primary_key,
                        is_foreign_key: col.is_foreign_key,
                        description: col.description,
                        max_length: col.max_length,
                        min_length: col.min_length,
                        database_table_id: DatabaseTableReference {
                            id: table_id.clone(),
                            name: table.name.clone(),
                            description: table.description.clone(),
                            table_reference: table.table_reference.clone(),
                            table_type: table.table_type.clone(),
                            entity_type: table.entity_type.clone(),
                            company_id: Some(table.company_id.clone()),
                            created_at: table.created_at.to_rfc3339(),
                            updated_at: table.updated_at.to_rfc3339(),
                        },
                        company_id: Some(table.company_id.clone()),
                        created_at: col.created_at.to_rfc3339(),
                        updated_at: col.updated_at.to_rfc3339(),
                    }
                }).collect())
            } else {
                None
            };

            entities.push(DatabaseTableEntity {
                id: table_id,
                name: table.name,
                description: table.description,
                table_reference: table.table_reference,
                table_type: table.table_type,
                entity_type: table.entity_type,
                resource: table.resource,
                company_id: Some(table.company_id),
                created_at: table.created_at.to_rfc3339(),
                updated_at: table.updated_at.to_rfc3339(),
                columns,
            });
        }

        Ok(PaginationResponse::new(
            "Tabelas de banco de dados recuperadas com sucesso",
            entities,
            total,
            page,
            limit,
        ))
    }

    pub async fn get_database_table_by_id(&self, id: &str) -> AppResult<DatabaseTableEntity> {
        let table = self.repository.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Database table not found".to_string()))?;

        Ok(DatabaseTableEntity {
            id: table.id.unwrap().to_hex(),
            name: table.name,
            description: table.description,
            table_reference: table.table_reference,
            table_type: table.table_type,
            entity_type: table.entity_type,
            resource: table.resource,
            company_id: Some(table.company_id),
            created_at: table.created_at.to_rfc3339(),
            updated_at: table.updated_at.to_rfc3339(),
            columns: None,
        })
    }


    pub async fn update_database_table(&self, id: &str, data: UpdateDatabaseTableDto) -> AppResult<DatabaseTableEntity> {
        let updated = self.repository.update(
            id,
            data.name,
            data.description,
            data.table_reference,
            data.table_type,
            data.entity_type,
            data.resource,
        ).await?;

        Ok(DatabaseTableEntity {
            id: updated.id.unwrap().to_hex(),
            name: updated.name,
            description: updated.description,
            table_reference: updated.table_reference,
            table_type: updated.table_type,
            entity_type: updated.entity_type,
            resource: updated.resource,
            company_id: Some(updated.company_id),
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
            columns: None,
        })
    }

    pub async fn delete_database_table(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }

    /// Fetch table columns from external database using stored connection configuration
    pub async fn get_table_columns_from_external_db(
        &self,
        connection_id: &str,
        table_name: &str,
        table_types: Option<Vec<String>>,
        entity_types: Option<Vec<String>>,
        table_references: Option<Vec<String>>,
    ) -> AppResult<serde_json::Value> {
        // Fetch table metadata from MongoDB
        let (tables, _, _) = self.repository.find_all(
            1,
            1,
            false,
            table_types,
            entity_types,
            table_references,
        ).await?;

        let table = tables.into_iter().next()
            .ok_or_else(|| AppError::NotFound("Database table not found in MongoDB".to_string()))?;

        let config_repo = self.config_repository.as_ref()
            .ok_or_else(|| AppError::DatabaseError("Database configuration repository not available".to_string()))?;

        let db_config = config_repo.find_by_id(connection_id).await?
            .ok_or_else(|| AppError::NotFound("Database configuration not found".to_string()))?;

        let username = db_config.username.as_ref().ok_or_else(|| AppError::BadRequest("Database username is required".to_string()))?;
        let password = db_config.password.as_ref().ok_or_else(|| AppError::BadRequest("Database password is required".to_string()))?;
        let port = db_config.port.ok_or_else(|| AppError::BadRequest("Database port is required".to_string()))?;
        let database = db_config.database.as_ref().ok_or_else(|| AppError::BadRequest("Database name is required".to_string()))?;

        let connection_string = format!(
            "oracle://{}:{}@{}:{}/{}",
            username,
            password,
            db_config.host,
            port,
            database
        );

        let connector = OracleConnector::new(&connection_string).await?;
        let columns = connector.get_table_columns(table_name).await?;

        drop(connector);

        // Map Oracle data types to simplified types
        let mapped_columns: Vec<serde_json::Value> = columns.iter().map(|col| {
            let data_type = col.get("dataType")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_uppercase();
            
            let simplified_type = match data_type.as_str() {
                "NUMBER" | "INTEGER" | "SMALLINT" | "NUMERIC" => "integer",
                "VARCHAR" | "VARCHAR2" | "CHAR" | "NVARCHAR2" | "NCHAR" | "CLOB" | "NCLOB" => "varchar",
                "DATE" => "date",
                "TIMESTAMP" | "TIMESTAMP(6)" => "timestamp",
                "BOOLEAN" => "boolean",
                _ => "varchar"
            };

            serde_json::json!({
                "id": col.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                "name": col.get("name").and_then(|v| v.as_str()).unwrap_or("").to_lowercase(),
                "dataType": simplified_type,
                "isNullable": col.get("isNullable").and_then(|v| v.as_bool()).unwrap_or(true)
            })
        }).collect();

        // Build the response structure with MongoDB table metadata + Oracle columns
        let response = serde_json::json!({
            "id": table.id.map(|id| id.to_hex()).unwrap_or_default(),
            "name": table.name,
            "description": table.description,
            "table_reference": table.table_reference,
            "table_type": table.table_type,
            "entityType": table.entity_type,
            "company_id": table.company_id,
            "created_at": table.created_at.to_rfc3339(),
            "updated_at": table.updated_at.to_rfc3339(),
            "columns": mapped_columns
        });
        
        Ok(response)
    }
}
