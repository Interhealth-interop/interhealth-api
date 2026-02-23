use std::sync::Arc;

use crate::domain::dtos::{CreateTargetIntegrationDto, TargetIntegrationEntity, UpdateTargetIntegrationDto};
use crate::infrastructure::repositories::{DatabaseViewRepository, TargetIntegrationRepository};
use crate::utils::{AppError, AppResult};

pub struct TargetIntegrationUseCase {
    repository: Arc<TargetIntegrationRepository>,
    view_repository: Arc<DatabaseViewRepository>,
}

impl TargetIntegrationUseCase {
    pub fn new(
        repository: Arc<TargetIntegrationRepository>,
        view_repository: Arc<DatabaseViewRepository>,
    ) -> Self {
        Self { repository, view_repository }
    }

    pub async fn create_target_integration(
        &self,
        data: CreateTargetIntegrationDto,
        company_id: String,
    ) -> AppResult<TargetIntegrationEntity> {
        let created = self
            .repository
            .create(
                data.name,
                data.version,
                data.host,
                data.auth_type,
                data.credentials,
                company_id.clone(),
            )
            .await?;

        let created_id = created
            .id
            .as_ref()
            .map(|id| id.to_hex())
            .unwrap_or_default();

        Ok(TargetIntegrationEntity {
            id: created_id,
            name: created.name,
            version: created.version,
            host: created.host,
            auth_type: created.auth_type,
            credentials: created.credentials,
            company_id: Some(created.company_id),
            created_at: created.created_at.to_rfc3339(),
            updated_at: created.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_target_integration_by_id(&self, id: &str) -> AppResult<TargetIntegrationEntity> {
        let target = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Target integration not found".to_string()))?;

        Ok(TargetIntegrationEntity {
            id: target.id.unwrap().to_hex(),
            name: target.name,
            version: target.version,
            host: target.host,
            auth_type: target.auth_type,
            credentials: target.credentials,
            company_id: Some(target.company_id),
            created_at: target.created_at.to_rfc3339(),
            updated_at: target.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_all_target_integrations(&self) -> AppResult<Vec<TargetIntegrationEntity>> {
        let targets = self.repository.find_all().await?;

        Ok(targets
            .into_iter()
            .map(|target| TargetIntegrationEntity {
                id: target.id.unwrap().to_hex(),
                name: target.name,
                version: target.version,
                host: target.host,
                auth_type: target.auth_type,
                credentials: target.credentials,
                company_id: Some(target.company_id),
                created_at: target.created_at.to_rfc3339(),
                updated_at: target.updated_at.to_rfc3339(),
            })
            .collect())
    }

    pub async fn get_target_integration_by_database_view_id(
        &self,
        database_view_id: &str,
    ) -> AppResult<TargetIntegrationEntity> {
        let view = self
            .view_repository
            .find_by_id(database_view_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Database view not found".to_string()))?;

        let target_integration_id = view
            .target_integration_id
            .ok_or_else(|| AppError::NotFound("Target integration not found for this database view".to_string()))?;

        let target = self
            .repository
            .find_by_id(&target_integration_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Target integration not found".to_string()))?;

        Ok(TargetIntegrationEntity {
            id: target.id.unwrap().to_hex(),
            name: target.name,
            version: target.version,
            host: target.host,
            auth_type: target.auth_type,
            credentials: target.credentials,
            company_id: Some(target.company_id),
            created_at: target.created_at.to_rfc3339(),
            updated_at: target.updated_at.to_rfc3339(),
        })
    }

    pub async fn update_target_integration(
        &self,
        id: &str,
        data: UpdateTargetIntegrationDto,
    ) -> AppResult<TargetIntegrationEntity> {
        let updated = self
            .repository
            .update(id, data.name, data.version, data.host, data.auth_type, data.credentials)
            .await?;

        Ok(TargetIntegrationEntity {
            id: updated.id.unwrap().to_hex(),
            name: updated.name,
            version: updated.version,
            host: updated.host,
            auth_type: updated.auth_type,
            credentials: updated.credentials,
            company_id: Some(updated.company_id),
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_target_integration(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }
}
