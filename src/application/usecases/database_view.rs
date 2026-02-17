use std::sync::Arc;

use crate::domain::dtos::{CreateDatabaseViewDto, UpdateDatabaseViewDto, DatabaseViewEntity, ResourceItemDto};
use crate::domain::entities::ResourceItem;
use crate::infrastructure::repositories::{DatabaseViewRepository, DatabaseConfigurationRepository};
use crate::utils::{AppError, AppResult, PaginationResponse};

pub struct DatabaseViewUseCase {
    repository: Arc<DatabaseViewRepository>,
    config_repository: Arc<DatabaseConfigurationRepository>,
}

impl DatabaseViewUseCase {
    pub fn new(repository: Arc<DatabaseViewRepository>, config_repository: Arc<DatabaseConfigurationRepository>) -> Self {
        Self { repository, config_repository }
    }

    fn convert_dto_to_entity_resources(dto_resources: Option<Vec<ResourceItemDto>>) -> Option<Vec<ResourceItem>> {
        dto_resources.map(|resources| {
            resources.into_iter().map(|r| ResourceItem {
                name: r.name,
                entity_type: r.entity_type,
                resource: r.resource,
            }).collect()
        })
    }

    fn convert_entity_to_dto_resources(entity_resources: Option<Vec<ResourceItem>>) -> Option<Vec<ResourceItemDto>> {
        entity_resources.map(|resources| {
            resources.into_iter().map(|r| ResourceItemDto {
                name: r.name,
                entity_type: r.entity_type,
                resource: r.resource,
            }).collect()
        })
    }

    pub async fn create_database_view(&self, data: CreateDatabaseViewDto, company_id: String) -> AppResult<DatabaseViewEntity> {
        let resources = Self::convert_dto_to_entity_resources(data.resources);
        
        let view = self.repository.create(
            data.name,
            data.description,
            data.reference.clone(),
            data.entity_type,
            data.main_resource.clone(),
            data.is_fhir_destination,
            data.is_interhealth_destination,
            data.database_configuration_id.clone(),
            company_id.clone(),
            resources,
        ).await?;

        Ok(DatabaseViewEntity {
            id: view.id.unwrap().to_hex(),
            name: view.name,
            description: view.description,
            reference: view.reference,
            entity_type: view.entity_type,
            main_resource: view.main_resource,
            is_fhir_destination: view.is_fhir_destination.unwrap_or(false),
            is_interhealth_destination: view.is_interhealth_destination.unwrap_or(false),
            database_configuration_id: data.database_configuration_id.clone(),
            company_id: Some(company_id),
            status: view.status,
            job_id: view.job_id,
            resources: Self::convert_entity_to_dto_resources(view.resources),
            started_at: view.started_at.map(|dt| dt.to_rfc3339()),
            cancelled_at: view.cancelled_at.map(|dt| dt.to_rfc3339()),
            created_at: view.created_at.to_rfc3339(),
            updated_at: view.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_all_database_views(&self, page: i64, limit: i64, database_configuration_id: Option<String>, order_field: Option<String>, order_by: Option<String>) -> AppResult<PaginationResponse<DatabaseViewEntity>> {
        use crate::utils::sort_helper::build_sort_document;
        
        let sort_document = build_sort_document(order_field, order_by);
        let (views, total) = self.repository.find_all(page, limit, database_configuration_id, sort_document).await?;

        let mut entities = Vec::new();
        for view in views {
            entities.push(DatabaseViewEntity {
                id: view.id.unwrap().to_hex(),
                name: view.name,
                description: view.description,
                reference: view.reference.clone(),
                entity_type: view.entity_type,
                main_resource: view.main_resource.clone(),
                is_fhir_destination: view.is_fhir_destination.unwrap_or(false),
                is_interhealth_destination: view.is_interhealth_destination.unwrap_or(false),
                database_configuration_id: view.database_configuration_id.clone(),
                company_id: Some(view.company_id),
                status: view.status,
                job_id: view.job_id,
                resources: Self::convert_entity_to_dto_resources(view.resources.clone()),
                started_at: view.started_at.map(|dt| dt.to_rfc3339()),
                cancelled_at: view.cancelled_at.map(|dt| dt.to_rfc3339()),
                created_at: view.created_at.to_rfc3339(),
                updated_at: view.updated_at.to_rfc3339(),
            });
        }

        Ok(PaginationResponse::new(
            "Database views retrieved successfully",
            entities,
            total,
            page,
            limit,
        ))
    }

    pub async fn get_database_view_by_id(&self, id: &str) -> AppResult<DatabaseViewEntity> {
        let view = self.repository.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Database view not found".to_string()))?;

        Ok(DatabaseViewEntity {
            id: view.id.unwrap().to_hex(),
            name: view.name.clone(),
            description: view.description.clone(),
            reference: view.reference.clone(),
            entity_type: view.entity_type.clone(),
            main_resource: view.main_resource.clone(),
            is_fhir_destination: view.is_fhir_destination.unwrap_or(false),
            is_interhealth_destination: view.is_interhealth_destination.unwrap_or(false),
            database_configuration_id: view.database_configuration_id.clone(),
            company_id: Some(view.company_id),
            status: view.status,
            job_id: view.job_id,
            resources: Self::convert_entity_to_dto_resources(view.resources),
            started_at: view.started_at.map(|dt| dt.to_rfc3339()),
            cancelled_at: view.cancelled_at.map(|dt| dt.to_rfc3339()),
            created_at: view.created_at.to_rfc3339(),
            updated_at: view.updated_at.to_rfc3339(),
        })
    }

    pub async fn update_database_view(&self, id: &str, data: UpdateDatabaseViewDto) -> AppResult<DatabaseViewEntity> {
        let resources = Self::convert_dto_to_entity_resources(data.resources);
        
        let updated = self.repository.update(
            id,
            data.name,
            data.description,
            data.reference,
            data.entity_type,
            data.main_resource,
            data.is_fhir_destination,
            data.is_interhealth_destination,
            data.database_configuration_id,
            data.status,
            resources,
        ).await?;

        Ok(DatabaseViewEntity {
            id: updated.id.unwrap().to_hex(),
            name: updated.name.clone(),
            description: updated.description.clone(),
            reference: updated.reference.clone(),
            entity_type: updated.entity_type.clone(),
            main_resource: updated.main_resource.clone(),
            is_fhir_destination: updated.is_fhir_destination.unwrap_or(false),
            is_interhealth_destination: updated.is_interhealth_destination.unwrap_or(false),
            database_configuration_id: updated.database_configuration_id.clone(),
            company_id: Some(updated.company_id),
            status: updated.status,
            job_id: updated.job_id,
            resources: Self::convert_entity_to_dto_resources(updated.resources),
            started_at: updated.started_at.map(|dt| dt.to_rfc3339()),
            cancelled_at: updated.cancelled_at.map(|dt| dt.to_rfc3339()),
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_database_view(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }

    pub async fn start_integration(&self, id: &str) -> AppResult<DatabaseViewEntity> {
        let view = self.repository.start_integration(id).await?;

        Ok(DatabaseViewEntity {
            id: view.id.unwrap().to_hex(),
            name: view.name,
            description: view.description,
            reference: view.reference.clone(),
            entity_type: view.entity_type,
            main_resource: view.main_resource.clone(),
            is_fhir_destination: view.is_fhir_destination.unwrap_or(false),
            is_interhealth_destination: view.is_interhealth_destination.unwrap_or(false),
            database_configuration_id: view.database_configuration_id.clone(),
            company_id: Some(view.company_id),
            status: view.status,
            job_id: view.job_id,
            resources: Self::convert_entity_to_dto_resources(view.resources),
            started_at: view.started_at.map(|dt| dt.to_rfc3339()),
            cancelled_at: view.cancelled_at.map(|dt| dt.to_rfc3339()),
            created_at: view.created_at.to_rfc3339(),
            updated_at: view.updated_at.to_rfc3339(),
        })
    }

    pub async fn cancel_integration(&self, id: &str) -> AppResult<DatabaseViewEntity> {
        let view = self.repository.cancel_integration(id).await?;

        Ok(DatabaseViewEntity {
            id: view.id.unwrap().to_hex(),
            name: view.name,
            description: view.description,
            reference: view.reference.clone(),
            entity_type: view.entity_type,
            main_resource: view.main_resource.clone(),
            is_fhir_destination: view.is_fhir_destination.unwrap_or(false),
            is_interhealth_destination: view.is_interhealth_destination.unwrap_or(false),
            database_configuration_id: view.database_configuration_id.clone(),
            company_id: Some(view.company_id),
            status: view.status,
            job_id: view.job_id,
            resources: Self::convert_entity_to_dto_resources(view.resources),
            started_at: view.started_at.map(|dt| dt.to_rfc3339()),
            cancelled_at: view.cancelled_at.map(|dt| dt.to_rfc3339()),
            created_at: view.created_at.to_rfc3339(),
            updated_at: view.updated_at.to_rfc3339(),
        })
    }
}
