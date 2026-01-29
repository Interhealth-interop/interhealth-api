use std::sync::Arc;

use crate::domain::dtos::{
    CreateDatabaseModelDto,
    UpdateDatabaseModelDto,
    DatabaseModelEntity,
    ModelValue as DtoModelValue,
};
use crate::domain::entities::ModelValue as EntityModelValue;
use crate::infrastructure::repositories::DatabaseModelRepository;
use crate::utils::{AppError, AppResult, PaginationResponse};

pub struct DatabaseModelUseCase {
    repository: Arc<DatabaseModelRepository>,
}

impl DatabaseModelUseCase {
    pub fn new(repository: Arc<DatabaseModelRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_database_model(&self, data: CreateDatabaseModelDto) -> AppResult<DatabaseModelEntity> {
        let entity_values: Vec<EntityModelValue> = data.values.into_iter()
            .map(|v| EntityModelValue {
                code: v.code,
                description: v.description,
            })
            .collect();

        let model = self.repository.create(
            data.name,
            data.type_field,
            data.description,
            entity_values,
        ).await?;

        let dto_values: Vec<DtoModelValue> = model.values.into_iter()
            .map(|v| DtoModelValue {
                code: v.code,
                description: v.description,
            })
            .collect();

        let values = if dto_values.is_empty() { None } else { Some(dto_values) };

        Ok(DatabaseModelEntity {
            id: model.id.unwrap().to_hex(),
            name: model.name,
            type_field: model.type_field,
            description: model.description,
            values,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_all_database_models(&self, page: i64, limit: i64, type_filter: Option<String>, include_values: bool) -> AppResult<PaginationResponse<DatabaseModelEntity>> {
        let (models, total) = self.repository.find_all(page, limit, type_filter).await?;

        let entities: Vec<DatabaseModelEntity> = models.into_iter().map(|m| {
            let dto_values: Vec<DtoModelValue> = m.values.into_iter()
                .map(|v| DtoModelValue {
                    code: v.code,
                    description: v.description,
                })
                .collect();

            let values = if include_values && !dto_values.is_empty() {
                Some(dto_values)
            } else {
                None
            };

            DatabaseModelEntity {
                id: m.id.unwrap().to_hex(),
                name: m.name,
                type_field: m.type_field,
                description: m.description,
                values,
                created_at: m.created_at.to_rfc3339(),
                updated_at: m.updated_at.to_rfc3339(),
            }
        }).collect();

        Ok(PaginationResponse::new(
            "Database models retrieved successfully",
            entities,
            total,
            page,
            limit,
        ))
    }

    pub async fn get_database_model_by_id(&self, id: &str) -> AppResult<DatabaseModelEntity> {
        let model = self.repository.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Database model not found".to_string()))?;

        let dto_values: Vec<DtoModelValue> = model.values.into_iter()
            .map(|v| DtoModelValue {
                code: v.code,
                description: v.description,
            })
            .collect();

        let values = if dto_values.is_empty() { None } else { Some(dto_values) };

        Ok(DatabaseModelEntity {
            id: model.id.unwrap().to_hex(),
            name: model.name,
            type_field: model.type_field,
            description: model.description,
            values,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        })
    }

    pub async fn update_database_model(&self, id: &str, data: UpdateDatabaseModelDto) -> AppResult<DatabaseModelEntity> {
        let entity_values = data.values.map(|values| {
            values.into_iter()
                .map(|v| EntityModelValue {
                    code: v.code,
                    description: v.description,
                })
                .collect()
        });

        let updated = self.repository.update(id, data.name, data.description, entity_values).await?;

        let dto_values: Vec<DtoModelValue> = updated.values.into_iter()
            .map(|v| DtoModelValue {
                code: v.code,
                description: v.description,
            })
            .collect();

        let values = if dto_values.is_empty() { None } else { Some(dto_values) };

        Ok(DatabaseModelEntity {
            id: updated.id.unwrap().to_hex(),
            name: updated.name,
            type_field: updated.type_field,
            description: updated.description,
            values,
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_database_model(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }
}
