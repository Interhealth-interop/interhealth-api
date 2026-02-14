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
        connection_id: Option<&str>,
        page: i64,
        limit: i64,
        order_field: Option<String>,
        order_by: Option<String>,
    ) -> AppResult<PaginationResponse<DatabaseModelValueEntity>> {
        use crate::utils::sort_helper::build_sort_document;
        
        let sort_document = build_sort_document(order_field, order_by);
        let (items, total) = self
            .repository
            .find_by_owner_with_default_and_company_custom(database_model_id, company_id, page, limit, sort_document)
            .await?;

        let entities: Vec<DatabaseModelValueEntity> = items
            .into_iter()
            .map(|v| {
                let company_hex = company_id.to_string();
                
                // Priority-based filtering:
                // 1. If connection_id is provided, return only the client with matching connection_id
                // 2. If no connection_id match, return client with matching company_id but no connection_id
                // 3. Otherwise, return empty clients array
                let filtered_clients: Vec<MappingValueItemEntity> = if let Some(conn_id) = connection_id {
                    // Try to find a client with matching connection_id
                    let connection_match = v
                        .clients
                        .iter()
                        .find(|c| {
                            c.company_id.to_hex() == company_hex &&
                            c.connection_id.as_ref().map(|id| id.to_hex()) == Some(conn_id.to_string())
                        });
                    
                    if let Some(client) = connection_match {
                        // Found a connection-specific mapping
                        vec![MappingValueItemEntity {
                            source_key: client.source_key.clone(),
                            source_description: client.source_description.clone(),
                            status: client.status.clone(),
                            company_id: client.company_id.to_hex(),
                            connection_id: client.connection_id.as_ref().map(|id| id.to_hex()),
                        }]
                    } else {
                        // No connection-specific mapping, try to find company-only mapping
                        let company_match = v
                            .clients
                            .iter()
                            .find(|c| {
                                c.company_id.to_hex() == company_hex &&
                                c.connection_id.is_none()
                            });
                        
                        if let Some(client) = company_match {
                            vec![MappingValueItemEntity {
                                source_key: client.source_key.clone(),
                                source_description: client.source_description.clone(),
                                status: client.status.clone(),
                                company_id: client.company_id.to_hex(),
                                connection_id: None,
                            }]
                        } else {
                            // No mapping found
                            vec![]
                        }
                    }
                } else {
                    // No connection_id provided, return company-only mapping
                    v.clients
                        .into_iter()
                        .filter(|c| c.company_id.to_hex() == company_hex && c.connection_id.is_none())
                        .map(|c| MappingValueItemEntity {
                            source_key: c.source_key,
                            source_description: c.source_description,
                            status: c.status,
                            company_id: c.company_id.to_hex(),
                            connection_id: None,
                        })
                        .collect()
                };

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
