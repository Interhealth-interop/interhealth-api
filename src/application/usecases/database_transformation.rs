use std::sync::Arc;

use crate::domain::dtos::{
    CreateDatabaseTransformationDto,
    UpdateDatabaseTransformationDto,
    DatabaseTransformationEntity,
    AvailableModelForTransformationDto,
};
use crate::infrastructure::repositories::{DatabaseTransformationRepository, DatabaseModelRepository};
use crate::utils::{AppError, AppResult, PaginationResponse};

pub struct DatabaseTransformationUseCase {
    repository: Arc<DatabaseTransformationRepository>,
}

impl DatabaseTransformationUseCase {
    pub fn new(repository: Arc<DatabaseTransformationRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_database_transformation(&self, data: CreateDatabaseTransformationDto) -> AppResult<DatabaseTransformationEntity> {
        let transformation = self.repository.create(
            data.name,
            data.type_field,
            data.company_id,
            data.value_mappings.clone(),
        ).await?;

        Ok(DatabaseTransformationEntity {
            id: transformation.id.unwrap().to_hex(),
            name: transformation.name,
            type_field: transformation.type_field,
            company_id: Some(transformation.company_id),
            value_mappings: Some(transformation.value_mappings),
            created_at: transformation.created_at.to_rfc3339(),
            updated_at: transformation.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_all_database_transformations(&self, page: i64, limit: i64, type_filter: Option<String>, include_values: bool) -> AppResult<PaginationResponse<DatabaseTransformationEntity>> {
        let (transformations, total) = self.repository.find_all(page, limit, type_filter).await?;

        let entities: Vec<DatabaseTransformationEntity> = transformations.into_iter().map(|t| {
            DatabaseTransformationEntity {
                id: t.id.unwrap().to_hex(),
                name: t.name,
                type_field: t.type_field,
                company_id: Some(t.company_id),
                value_mappings: if include_values { Some(t.value_mappings) } else { None },
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            }
        }).collect();

        Ok(PaginationResponse::new(
            "Database transformations retrieved successfully",
            entities,
            total,
            page,
            limit,
        ))
    }

    pub async fn get_database_transformation_by_id(&self, id: &str) -> AppResult<DatabaseTransformationEntity> {
        let transformation = self.repository.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Database transformation not found".to_string()))?;

        Ok(DatabaseTransformationEntity {
            id: transformation.id.unwrap().to_hex(),
            name: transformation.name,
            type_field: transformation.type_field,
            company_id: Some(transformation.company_id),
            value_mappings: Some(transformation.value_mappings),
            created_at: transformation.created_at.to_rfc3339(),
            updated_at: transformation.updated_at.to_rfc3339(),
        })
    }

    pub async fn update_database_transformation(&self, id: &str, data: UpdateDatabaseTransformationDto) -> AppResult<DatabaseTransformationEntity> {
        let updated = self.repository.update(id, data.name, data.value_mappings).await?;

        Ok(DatabaseTransformationEntity {
            id: updated.id.unwrap().to_hex(),
            name: updated.name,
            type_field: updated.type_field,
            company_id: Some(updated.company_id),
            value_mappings: Some(updated.value_mappings),
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_database_transformation(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }

    pub async fn get_available_models_for_transformation(
        &self,
        database_model_repository: Arc<DatabaseModelRepository>,
        page: i64,
        limit: i64,
        type_filter: Option<String>,
    ) -> AppResult<PaginationResponse<AvailableModelForTransformationDto>> {
        let (all_models, total_models) = database_model_repository.find_all(1, 10000, type_filter.clone()).await?;
        let (all_transformations, _) = self.repository.find_all(1, 1000, type_filter).await?;

        let transformed_types: std::collections::HashSet<String> = all_transformations
            .iter()
            .map(|t| t.type_field.clone())
            .collect();

        let mut all_items: Vec<AvailableModelForTransformationDto> = all_models
            .into_iter()
            .filter_map(|m| {
                m.id.map(|id| {
                    let is_transformed = transformed_types.contains(&m.type_field);
                    AvailableModelForTransformationDto {
                        id: id.to_hex(),
                        name: m.name,
                        type_field: m.type_field,
                        description: m.description,
                        is_transformed,
                        created_at: m.created_at.to_rfc3339(),
                        updated_at: m.updated_at.to_rfc3339(),
                    }
                })
            })
            .collect();

        let custom_transformations: Vec<AvailableModelForTransformationDto> = all_transformations
            .into_iter()
            .filter(|t| t.type_field.to_uppercase() == "CUSTOM")
            .filter_map(|t| {
                t.id.map(|id| {
                    AvailableModelForTransformationDto {
                        id: id.to_hex(),
                        name: t.name,
                        type_field: t.type_field,
                        description: "Custom transformation".to_string(),
                        is_transformed: true,
                        created_at: t.created_at.to_rfc3339(),
                        updated_at: t.updated_at.to_rfc3339(),
                    }
                })
            })
            .collect();

        all_items.extend(custom_transformations);
        
        let total_count = all_items.len() as i64;
        let start_index = ((page - 1) * limit) as usize;
        let end_index = (start_index + limit as usize).min(all_items.len());
        
        let paginated_items = if start_index < all_items.len() {
            all_items[start_index..end_index].to_vec()
        } else {
            Vec::new()
        };

        Ok(PaginationResponse::new(
            "Available models for transformation retrieved successfully",
            paginated_items,
            total_count,
            page,
            limit,
        ))
    }
}
