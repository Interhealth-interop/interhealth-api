use std::sync::Arc;

use crate::domain::dtos::{DatabaseModelValueEntity, MappingValueItemEntity};
use crate::infrastructure::repositories::DatabaseModelValueRepository;
use crate::utils::{AppResult, PaginationResponse};

pub struct MappingValueUseCase {
    repository: Arc<DatabaseModelValueRepository>,
}

impl MappingValueUseCase {
    pub fn new(repository: Arc<DatabaseModelValueRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_mapping_values_by_database_model_and_company(
        &self,
        database_model_id: &str,
        company_id: &str,
        page: i64,
        limit: i64,
    ) -> AppResult<PaginationResponse<DatabaseModelValueEntity>> {
        let (items, total) = self
            .repository
            .find_by_owner_with_default_and_company_custom(database_model_id, company_id, page, limit)
            .await?;

        let entities: Vec<DatabaseModelValueEntity> = items
            .into_iter()
            .map(|v| {
                let company_hex = company_id.to_string();
                let filtered_clients: Vec<MappingValueItemEntity> = v
                    .clients
                    .into_iter()
                    .filter(|c| c.company_id.to_hex() == company_hex)
                    .map(|c| MappingValueItemEntity {
                        source_key: c.source_key,
                        source_description: c.source_description,
                        status: c.status,
                        company_id: c.company_id.to_hex(),
                        connection_id: c.connection_id.map(|id| id.to_hex()),
                    })
                    .collect();

                DatabaseModelValueEntity {
                    id: v.id.unwrap().to_hex(),
                    owner_id: v.owner_id.to_hex(),
                    type_field: v.type_field,
                    code: v.code,
                    description: v.description,
                    clients: filtered_clients,
                    created_at: v.created_at.to_rfc3339(),
                    updated_at: v.updated_at.to_rfc3339(),
                }
            })
            .collect();

        Ok(PaginationResponse::new(
            "Mapping values retrieved successfully",
            entities,
            total,
            page,
            limit,
        ))
    }
}
