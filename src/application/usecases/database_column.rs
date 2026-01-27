use std::sync::Arc;

use crate::domain::dtos::{CreateDatabaseColumnDto, UpdateDatabaseColumnDto, DatabaseColumnEntity};
use crate::infrastructure::repositories::DatabaseColumnRepository;
use crate::utils::{AppError, AppResult, PaginationResponse};

pub struct DatabaseColumnUseCase {
    repository: Arc<DatabaseColumnRepository>,
}

impl DatabaseColumnUseCase {
    pub fn new(repository: Arc<DatabaseColumnRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_database_column(&self, data: CreateDatabaseColumnDto, company_id: String) -> AppResult<DatabaseColumnEntity> {
        let column = self.repository.create(
            data.name.clone(),
            data.reference.clone(),
            data.data_type.clone(),
            data.is_nullable,
            data.is_primary_key,
            data.is_foreign_key,
            data.description.clone(),
            data.max_length,
            data.min_length,
            data.database_table_id.clone(),
            company_id.clone(),
        ).await?;

        Ok(DatabaseColumnEntity {
            id: column.id.unwrap().to_hex(),
            name: column.name,
            reference: column.reference,
            data_type: column.data_type,
            is_nullable: column.is_nullable,
            is_primary_key: column.is_primary_key,
            is_foreign_key: column.is_foreign_key,
            description: column.description,
            max_length: column.max_length,
            min_length: column.min_length,
            database_table_id: crate::domain::dtos::DatabaseTableReference {
                id: data.database_table_id,
                name: String::new(),
                description: String::new(),
                table_reference: Some(String::new()),
                table_type: Some(String::new()),
                entity_type: String::new(),
                company_id: Some(company_id.clone()),
                created_at: column.created_at.to_rfc3339(),
                updated_at: column.updated_at.to_rfc3339(),
            },
            company_id: Some(company_id),
            created_at: column.created_at.to_rfc3339(),
            updated_at: column.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_all_database_columns(&self, page: i64, limit: i64) -> AppResult<PaginationResponse<DatabaseColumnEntity>> {
        let (columns, total) = self.repository.find_all(page, limit).await?;

        let entities: Vec<DatabaseColumnEntity> = columns.into_iter().map(|column| {
            DatabaseColumnEntity {
                id: column.id.unwrap().to_hex(),
                name: column.name.clone(),
                reference: column.reference.clone(),
                data_type: column.data_type.clone(),
                is_nullable: column.is_nullable,
                is_primary_key: column.is_primary_key,
                is_foreign_key: column.is_foreign_key,
                description: column.description.clone(),
                max_length: column.max_length,
                min_length: column.min_length,
                database_table_id: crate::domain::dtos::DatabaseTableReference {
                    id: column.database_table_id.clone(),
                    name: String::new(),
                    description: String::new(),
                    table_reference: Some(String::new()),
                    table_type: Some(String::new()),
                    entity_type: String::new(),
                    company_id: Some(column.company_id.clone()),
                    created_at: column.created_at.to_rfc3339(),
                    updated_at: column.updated_at.to_rfc3339(),
                },
                company_id: Some(column.company_id),
                created_at: column.created_at.to_rfc3339(),
                updated_at: column.updated_at.to_rfc3339(),
            }
        }).collect();

        Ok(PaginationResponse::new(
            "Colunas de banco de dados recuperadas com sucesso",
            entities,
            total,
            page,
            limit,
        ))
    }

    pub async fn get_database_column_by_id(&self, id: &str) -> AppResult<DatabaseColumnEntity> {
        let column = self.repository.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Database column not found".to_string()))?;

        Ok(DatabaseColumnEntity {
            id: column.id.unwrap().to_hex(),
            name: column.name.clone(),
            reference: column.reference.clone(),
            data_type: column.data_type.clone(),
            is_nullable: column.is_nullable,
            is_primary_key: column.is_primary_key,
            is_foreign_key: column.is_foreign_key,
            description: column.description.clone(),
            max_length: column.max_length,
            min_length: column.min_length,
            database_table_id: crate::domain::dtos::DatabaseTableReference {
                id: column.database_table_id.clone(),
                name: String::new(),
                description: String::new(),
                table_reference: Some(String::new()),
                table_type: Some(String::new()),
                entity_type: String::new(),
                company_id: Some(column.company_id.clone()),
                created_at: column.created_at.to_rfc3339(),
                updated_at: column.updated_at.to_rfc3339(),
            },
            company_id: Some(column.company_id),
            created_at: column.created_at.to_rfc3339(),
            updated_at: column.updated_at.to_rfc3339(),
        })
    }


    pub async fn get_database_columns_by_table_id(&self, table_id: &str) -> AppResult<Vec<DatabaseColumnEntity>> {
        let columns = self.repository.find_by_table_id(table_id).await?;

        let entities: Vec<DatabaseColumnEntity> = columns.into_iter().map(|column| {
            DatabaseColumnEntity {
                id: column.id.unwrap().to_hex(),
                name: column.name.clone(),
                reference: column.reference.clone(),
                data_type: column.data_type.clone(),
                is_nullable: column.is_nullable,
                is_primary_key: column.is_primary_key,
                is_foreign_key: column.is_foreign_key,
                description: column.description.clone(),
                max_length: column.max_length,
                min_length: column.min_length,
                database_table_id: crate::domain::dtos::DatabaseTableReference {
                    id: column.database_table_id.clone(),
                    name: String::new(),
                    description: String::new(),
                    table_reference: Some(String::new()),
                    table_type: Some(String::new()),
                    entity_type: String::new(),
                    company_id: Some(column.company_id.clone()),
                    created_at: column.created_at.to_rfc3339(),
                    updated_at: column.updated_at.to_rfc3339(),
                },
                company_id: Some(column.company_id),
                created_at: column.created_at.to_rfc3339(),
                updated_at: column.updated_at.to_rfc3339(),
            }
        }).collect();

        Ok(entities)
    }

    pub async fn update_database_column(&self, id: &str, data: UpdateDatabaseColumnDto) -> AppResult<DatabaseColumnEntity> {
        let updated = self.repository.update(
            id,
            data.name,
            data.reference.clone(),
            data.data_type,
            data.is_nullable,
            data.is_primary_key,
            data.is_foreign_key,
            data.description,
            data.max_length,
            data.min_length,
        ).await?;

        Ok(DatabaseColumnEntity {
            id: updated.id.unwrap().to_hex(),
            name: updated.name.clone(),
            reference: updated.reference.clone(),
            data_type: updated.data_type.clone(),
            is_nullable: updated.is_nullable,
            is_primary_key: updated.is_primary_key,
            is_foreign_key: updated.is_foreign_key,
            description: updated.description.clone(),
            max_length: updated.max_length,
            min_length: updated.min_length,
            database_table_id: crate::domain::dtos::DatabaseTableReference {
                id: updated.database_table_id.clone(),
                name: String::new(),
                description: String::new(),
                table_reference: Some(String::new()),
                table_type: Some(String::new()),
                entity_type: String::new(),
                company_id: Some(updated.company_id.clone()),
                created_at: updated.created_at.to_rfc3339(),
                updated_at: updated.updated_at.to_rfc3339(),
            },
            company_id: Some(updated.company_id),
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_database_column(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }
}
