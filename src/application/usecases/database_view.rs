use std::sync::Arc;

use crate::domain::dtos::{CreateDatabaseViewDto, UpdateDatabaseViewDto, DatabaseViewEntity, ResourceItemDto};
use crate::domain::entities::ResourceItem;
use crate::infrastructure::repositories::{DatabaseViewRepository, DatabaseConfigurationRepository, DatabaseViewMappingRepository};
use crate::utils::{AppError, AppResult, PaginationResponse};

pub struct DatabaseViewUseCase {
    repository: Arc<DatabaseViewRepository>,
    config_repository: Arc<DatabaseConfigurationRepository>,
    mapping_repository: Arc<DatabaseViewMappingRepository>,
}

impl DatabaseViewUseCase {
    pub fn new(
        repository: Arc<DatabaseViewRepository>,
        config_repository: Arc<DatabaseConfigurationRepository>,
        mapping_repository: Arc<DatabaseViewMappingRepository>,
    ) -> Self {
        Self { repository, config_repository, mapping_repository }
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
            data.resource.clone(),
            data.entity_type,
            data.main_resource.clone(),
            data.is_fhir_destination,
            data.is_interhealth_destination,
            data.database_configuration_id.clone(),
            company_id.clone(),
            resources,
        ).await?;

        if let Some(target_integration_id) = data.target_integration_id.clone() {
            self
                .repository
                .set_target_integration_id(&view.id.as_ref().unwrap().to_hex(), Some(target_integration_id))
                .await?;
        }

        let refreshed = self
            .repository
            .find_by_id(&view.id.as_ref().unwrap().to_hex())
            .await?
            .ok_or_else(|| AppError::NotFound("Database view not found after create".to_string()))?;

        Ok(DatabaseViewEntity {
            id: refreshed.id.unwrap().to_hex(),
            name: refreshed.name,
            description: refreshed.description,
            resource: refreshed.resource,
            entity_type: refreshed.entity_type,
            main_resource: refreshed.main_resource,
            is_fhir_destination: refreshed.is_fhir_destination.unwrap_or(false),
            is_interhealth_destination: refreshed.is_interhealth_destination.unwrap_or(false),
            database_configuration_id: data.database_configuration_id.clone(),
            target_integration_id: refreshed.target_integration_id.clone(),
            company_id: Some(company_id),
            status: refreshed.status,
            job_id: refreshed.job_id,
            resources: Self::convert_entity_to_dto_resources(refreshed.resources),
            started_at: refreshed.started_at.map(|dt| dt.to_rfc3339()),
            cancelled_at: refreshed.cancelled_at.map(|dt| dt.to_rfc3339()),
            created_at: refreshed.created_at.to_rfc3339(),
            updated_at: refreshed.updated_at.to_rfc3339(),
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
                resource: view.resource.clone(),
                entity_type: view.entity_type,
                main_resource: view.main_resource.clone(),
                is_fhir_destination: view.is_fhir_destination.unwrap_or(false),
                is_interhealth_destination: view.is_interhealth_destination.unwrap_or(false),
                database_configuration_id: view.database_configuration_id.clone(),
                target_integration_id: view.target_integration_id.clone(),
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
            resource: view.resource.clone(),
            entity_type: view.entity_type.clone(),
            main_resource: view.main_resource.clone(),
            is_fhir_destination: view.is_fhir_destination.unwrap_or(false),
            is_interhealth_destination: view.is_interhealth_destination.unwrap_or(false),
            database_configuration_id: view.database_configuration_id.clone(),
            target_integration_id: view.target_integration_id.clone(),
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
            data.resource,
            data.entity_type,
            data.main_resource,
            data.is_fhir_destination,
            data.is_interhealth_destination,
            data.database_configuration_id,
            data.status,
            resources,
        ).await?;

        if let Some(target_integration_id) = data.target_integration_id.clone() {
            self
                .repository
                .set_target_integration_id(id, Some(target_integration_id))
                .await?;
        }

        let refreshed = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Database view not found after update".to_string()))?;

        Ok(DatabaseViewEntity {
            id: refreshed.id.unwrap().to_hex(),
            name: refreshed.name.clone(),
            description: refreshed.description.clone(),
            resource: refreshed.resource.clone(),
            entity_type: refreshed.entity_type.clone(),
            main_resource: refreshed.main_resource.clone(),
            is_fhir_destination: refreshed.is_fhir_destination.unwrap_or(false),
            is_interhealth_destination: refreshed.is_interhealth_destination.unwrap_or(false),
            database_configuration_id: refreshed.database_configuration_id.clone(),
            target_integration_id: refreshed.target_integration_id.clone(),
            company_id: Some(refreshed.company_id),
            status: refreshed.status,
            job_id: refreshed.job_id,
            resources: Self::convert_entity_to_dto_resources(refreshed.resources),
            started_at: refreshed.started_at.map(|dt| dt.to_rfc3339()),
            cancelled_at: refreshed.cancelled_at.map(|dt| dt.to_rfc3339()),
            created_at: refreshed.created_at.to_rfc3339(),
            updated_at: refreshed.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_database_view(&self, id: &str) -> AppResult<bool> {
        self.mapping_repository.delete_by_data_view_id(id).await?;
        self.repository.delete(id).await
    }

    pub async fn start_integration(&self, id: &str) -> AppResult<DatabaseViewEntity> {
        let view = self.repository.start_integration(id).await?;

        Ok(DatabaseViewEntity {
            id: view.id.unwrap().to_hex(),
            name: view.name,
            description: view.description,
            resource: view.resource.clone(),
            entity_type: view.entity_type,
            main_resource: view.main_resource.clone(),
            is_fhir_destination: view.is_fhir_destination.unwrap_or(false),
            is_interhealth_destination: view.is_interhealth_destination.unwrap_or(false),
            database_configuration_id: view.database_configuration_id.clone(),
            target_integration_id: view.target_integration_id.clone(),
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
            resource: view.resource.clone(),
            entity_type: view.entity_type,
            main_resource: view.main_resource.clone(),
            is_fhir_destination: view.is_fhir_destination.unwrap_or(false),
            is_interhealth_destination: view.is_interhealth_destination.unwrap_or(false),
            database_configuration_id: view.database_configuration_id.clone(),
            target_integration_id: view.target_integration_id.clone(),
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
