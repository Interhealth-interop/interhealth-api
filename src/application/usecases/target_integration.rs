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
        if self
            .repository
            .find_by_database_view_id(&data.database_view_id)
            .await?
            .is_some()
        {
            return Err(AppError::BadRequest(
                "Target integration already exists for this databaseViewId".to_string(),
            ));
        }

        let created = self
            .repository
            .create(
                data.name,
                data.version,
                data.host,
                data.auth_type,
                data.credentials,
                data.database_view_id.clone(),
                company_id.clone(),
            )
            .await?;

        let created_id = created
            .id
            .as_ref()
            .map(|id| id.to_hex())
            .unwrap_or_default();

        self
            .view_repository
            .set_target_integration_id(&data.database_view_id, Some(created_id.clone()))
            .await?;

        Ok(TargetIntegrationEntity {
            id: created_id,
            name: created.name,
            version: created.version,
            host: created.host,
            auth_type: created.auth_type,
            credentials: created.credentials,
            database_view_id: created.database_view_id,
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
            database_view_id: target.database_view_id,
            company_id: Some(target.company_id),
            created_at: target.created_at.to_rfc3339(),
            updated_at: target.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_target_integration_by_database_view_id(
        &self,
        database_view_id: &str,
    ) -> AppResult<TargetIntegrationEntity> {
        let target = self
            .repository
            .find_by_database_view_id(database_view_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Target integration not found".to_string()))?;

        Ok(TargetIntegrationEntity {
            id: target.id.unwrap().to_hex(),
            name: target.name,
            version: target.version,
            host: target.host,
            auth_type: target.auth_type,
            credentials: target.credentials,
            database_view_id: target.database_view_id,
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
            database_view_id: updated.database_view_id,
            company_id: Some(updated.company_id),
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_target_integration(&self, id: &str) -> AppResult<bool> {
        let existing = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Target integration not found".to_string()))?;

        let deleted = self.repository.delete(id).await?;
        if deleted {
            self
                .view_repository
                .set_target_integration_id(&existing.database_view_id, None)
                .await?;
        }

        Ok(deleted)
    }
}
