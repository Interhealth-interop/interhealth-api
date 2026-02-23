use std::sync::Arc;

use crate::domain::dtos::{
    CreateIntegrationControlDto, IntegrationControlEntity, UpdateIntegrationControlDto,
};
use crate::infrastructure::repositories::IntegrationControlRepository;
use crate::utils::{AppError, AppResult};

pub struct IntegrationControlUseCase {
    repository: Arc<IntegrationControlRepository>,
}

impl IntegrationControlUseCase {
    pub fn new(repository: Arc<IntegrationControlRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_integration_control(
        &self,
        data: CreateIntegrationControlDto,
        company_id: String,
    ) -> AppResult<IntegrationControlEntity> {
        let created = self
            .repository
            .create(
                data.name,
                data.database_view_id,
                data.cron,
                data.date_field,
                data.start_at,
                data.end_at,
                data.control_field.unwrap_or_else(|| String::new()),
                company_id,
            )
            .await?;

        Ok(IntegrationControlEntity {
            id: created.id.unwrap().to_hex(),
            name: created.name,
            database_view_id: created.database_view_id,
            cron: created.cron,
            date_field: created.date_field,
            start_at: created.start_at.map(|dt| dt.to_rfc3339()),
            end_at: created.end_at.map(|dt| dt.to_rfc3339()),
            last_run_at: created.last_run_at.map(|dt| dt.to_rfc3339()),
            control_field: created.control_field,
            company_id: Some(created.company_id),
            created_at: created.created_at.to_rfc3339(),
            updated_at: created.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_integration_control_by_id(&self, id: &str) -> AppResult<IntegrationControlEntity> {
        let control = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Integration control not found".to_string()))?;

        Ok(IntegrationControlEntity {
            id: control.id.unwrap().to_hex(),
            name: control.name,
            database_view_id: control.database_view_id,
            cron: control.cron,
            date_field: control.date_field,
            start_at: control.start_at.map(|dt| dt.to_rfc3339()),
            end_at: control.end_at.map(|dt| dt.to_rfc3339()),
            last_run_at: control.last_run_at.map(|dt| dt.to_rfc3339()),
            control_field: control.control_field,
            company_id: Some(control.company_id),
            created_at: control.created_at.to_rfc3339(),
            updated_at: control.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_integration_controls_by_database_view_id(
        &self,
        database_view_id: &str,
    ) -> AppResult<Vec<IntegrationControlEntity>> {
        let controls = self.repository.find_by_database_view_id(database_view_id).await?;

        Ok(controls
            .into_iter()
            .map(|control| IntegrationControlEntity {
                id: control.id.unwrap().to_hex(),
                name: control.name,
                database_view_id: control.database_view_id,
                cron: control.cron,
                date_field: control.date_field,
                start_at: control.start_at.map(|dt| dt.to_rfc3339()),
                end_at: control.end_at.map(|dt| dt.to_rfc3339()),
                last_run_at: control.last_run_at.map(|dt| dt.to_rfc3339()),
                control_field: control.control_field,
                company_id: Some(control.company_id),
                created_at: control.created_at.to_rfc3339(),
                updated_at: control.updated_at.to_rfc3339(),
            })
            .collect())
    }

    pub async fn update_integration_control(
        &self,
        id: &str,
        data: UpdateIntegrationControlDto,
    ) -> AppResult<IntegrationControlEntity> {
        let updated = self
            .repository
            .update(
                id,
                data.name,
                data.cron,
                data.date_field,
                data.start_at,
                data.end_at,
                data.control_field,
            )
            .await?;

        Ok(IntegrationControlEntity {
            id: updated.id.unwrap().to_hex(),
            name: updated.name,
            database_view_id: updated.database_view_id,
            cron: updated.cron,
            date_field: updated.date_field,
            start_at: updated.start_at.map(|dt| dt.to_rfc3339()),
            end_at: updated.end_at.map(|dt| dt.to_rfc3339()),
            last_run_at: updated.last_run_at.map(|dt| dt.to_rfc3339()),
            control_field: updated.control_field,
            company_id: Some(updated.company_id),
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_integration_control(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }
}
