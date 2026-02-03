use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::domain::entities::{FieldMapping, DatabaseTransformation, DatabaseModelValue};
use crate::infrastructure::repositories::{DatabaseViewMappingRepository, DatabaseViewRepository, DatabaseConfigurationRepository, DatabaseTableRepository, DatabaseTransformationRepository, DatabaseModelValueRepository};
use crate::infrastructure::adapters::oracledb::OracleConnector;
use crate::utils::{AppError, AppResult, PaginationResponse, Replacer, Validator};
use super::fhir::FhirGenerator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatabaseViewMappingDto {
    pub name: String,
    pub description: String,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "databaseTableOriginId")]
    pub database_table_origin_id: String,
    #[serde(rename = "databaseTableDestinyId")]
    pub database_table_destiny_id: String,
    #[serde(rename = "dataViewId")]
    pub data_view_id: String,
    #[serde(rename = "fieldMappings")]
    pub field_mappings: Vec<FieldMapping>,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "draft".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatabaseViewMappingDto {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: Option<String>,
    #[serde(rename = "databaseTableOriginId")]
    pub database_table_origin_id: Option<String>,
    #[serde(rename = "databaseTableDestinyId")]
    pub database_table_destiny_id: Option<String>,
    #[serde(rename = "dataViewId")]
    pub data_view_id: Option<String>,
    #[serde(rename = "fieldMappings")]
    pub field_mappings: Option<Vec<FieldMapping>>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseViewMappingEntity {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "databaseTableOriginId")]
    pub database_table_origin_id: String,
    #[serde(rename = "databaseTableDestinyId")]
    pub database_table_destiny_id: String,
    #[serde(rename = "dataViewId")]
    pub data_view_id: String,
    #[serde(rename = "fieldMappings")]
    pub field_mappings: Vec<FieldMapping>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct DatabaseViewMappingUseCase {
    repository: Arc<DatabaseViewMappingRepository>,
    view_repository: Option<Arc<DatabaseViewRepository>>,
    config_repository: Option<Arc<DatabaseConfigurationRepository>>,
    table_repository: Option<Arc<DatabaseTableRepository>>,
    transformation_repository: Option<Arc<DatabaseTransformationRepository>>,
    model_value_repository: Option<Arc<DatabaseModelValueRepository>>,
}

impl DatabaseViewMappingUseCase {
    pub fn new(repository: Arc<DatabaseViewMappingRepository>) -> Self {
        Self { 
            repository,
            view_repository: None,
            config_repository: None,
            table_repository: None,
            transformation_repository: None,
            model_value_repository: None,
        }
    }

    pub fn with_repositories(
        repository: Arc<DatabaseViewMappingRepository>,
        view_repository: Arc<DatabaseViewRepository>,
        config_repository: Arc<DatabaseConfigurationRepository>,
        table_repository: Arc<DatabaseTableRepository>,
        transformation_repository: Arc<DatabaseTransformationRepository>,
        model_value_repository: Arc<DatabaseModelValueRepository>,
    ) -> Self {
        Self {
            repository,
            view_repository: Some(view_repository),
            config_repository: Some(config_repository),
            table_repository: Some(table_repository),
            transformation_repository: Some(transformation_repository),
            model_value_repository: Some(model_value_repository),
        }
    }

    // Helper method to convert resource type references to UUID references for bundles
    fn convert_references_to_uuid(resource: &mut Value, uuid_map: &std::collections::HashMap<String, String>) {
        if let Some(resource_obj) = resource.get_mut("resource") {
            Self::convert_references_recursive(resource_obj, uuid_map);
        }
    }

    fn convert_references_recursive(value: &mut Value, uuid_map: &std::collections::HashMap<String, String>) {
        match value {
            Value::Object(map) => {
                // Check if this object has a "reference" field
                if let Some(reference) = map.get_mut("reference") {
                    if let Some(ref_str) = reference.as_str() {
                        // Check if it's a resource type reference (e.g., "Patient/123")
                        if let Some(slash_pos) = ref_str.find('/') {
                            let resource_type = &ref_str[..slash_pos];
                            // If we have a UUID mapping for this resource type, replace it
                            if let Some(uuid) = uuid_map.get(resource_type) {
                                *reference = json!(format!("urn:uuid:{}", uuid));
                            }
                        }
                    }
                }
                
                // Recursively process all values in the object
                for (_, v) in map.iter_mut() {
                    Self::convert_references_recursive(v, uuid_map);
                }
            }
            Value::Array(arr) => {
                // Recursively process all items in the array
                for item in arr.iter_mut() {
                    Self::convert_references_recursive(item, uuid_map);
                }
            }
            _ => {}
        }
    }

    pub async fn create_database_view_mapping(&self, data: CreateDatabaseViewMappingDto, _company_id: String) -> AppResult<DatabaseViewMappingEntity> {
        let mapping = self.repository.create(
            data.name,
            data.description,
            data.entity_type,
            data.database_table_origin_id,
            data.database_table_destiny_id,
            data.data_view_id,
            data.field_mappings,
            data.status,
        ).await?;

        Ok(DatabaseViewMappingEntity {
            id: mapping.id.unwrap().to_hex(),
            name: mapping.name,
            description: mapping.description,
            entity_type: mapping.entity_type,
            database_table_origin_id: mapping.database_table_origin_id,
            database_table_destiny_id: mapping.database_table_destiny_id,
            data_view_id: mapping.data_view_id,
            field_mappings: mapping.field_mappings,
            status: mapping.status,
            created_at: mapping.created_at.to_rfc3339(),
            updated_at: mapping.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_all_database_view_mappings(
        &self,
        page: i64,
        limit: i64,
        database_table_origin_id: Option<String>,
        database_table_destiny_id: Option<String>,
        entity_type: Option<String>,
        data_view_id: Option<String>,
    ) -> AppResult<PaginationResponse<DatabaseViewMappingEntity>> {
        let (mappings, total, _pages) = self.repository.find_all(
            page,
            limit,
            database_table_origin_id,
            database_table_destiny_id,
            entity_type,
            data_view_id,
        ).await?;

        let entities: Vec<DatabaseViewMappingEntity> = mappings.into_iter().map(|m| {
            DatabaseViewMappingEntity {
                id: m.id.unwrap().to_hex(),
                name: m.name,
                description: m.description,
                entity_type: m.entity_type,
                database_table_origin_id: m.database_table_origin_id,
                database_table_destiny_id: m.database_table_destiny_id,
                data_view_id: m.data_view_id,
                field_mappings: m.field_mappings,
                status: m.status,
                created_at: m.created_at.to_rfc3339(),
                updated_at: m.updated_at.to_rfc3339(),
            }
        }).collect();

        Ok(PaginationResponse::new(
            "Database view mappings retrieved successfully",
            entities,
            total,
            page,
            limit,
        ))
    }

    pub async fn get_database_view_mapping_by_id(&self, id: &str) -> AppResult<DatabaseViewMappingEntity> {
        let mapping = self.repository.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Database view mapping not found".to_string()))?;

        Ok(DatabaseViewMappingEntity {
            id: mapping.id.unwrap().to_hex(),
            name: mapping.name,
            description: mapping.description,
            entity_type: mapping.entity_type,
            database_table_origin_id: mapping.database_table_origin_id,
            database_table_destiny_id: mapping.database_table_destiny_id,
            data_view_id: mapping.data_view_id,
            field_mappings: mapping.field_mappings,
            status: mapping.status,
            created_at: mapping.created_at.to_rfc3339(),
            updated_at: mapping.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_database_view_mapping_by_name(&self, name: &str) -> AppResult<DatabaseViewMappingEntity> {
        let mapping = self.repository.find_by_name(name).await?
            .ok_or_else(|| AppError::NotFound("Database view mapping not found".to_string()))?;

        Ok(DatabaseViewMappingEntity {
            id: mapping.id.unwrap().to_hex(),
            name: mapping.name,
            description: mapping.description,
            entity_type: mapping.entity_type,
            database_table_origin_id: mapping.database_table_origin_id,
            database_table_destiny_id: mapping.database_table_destiny_id,
            data_view_id: mapping.data_view_id,
            field_mappings: mapping.field_mappings,
            status: mapping.status,
            created_at: mapping.created_at.to_rfc3339(),
            updated_at: mapping.updated_at.to_rfc3339(),
        })
    }

    pub async fn get_database_view_mappings_by_data_view_id(&self, data_view_id: &str) -> AppResult<Vec<DatabaseViewMappingEntity>> {
        let mappings = self.repository.find_by_data_view_id(data_view_id).await?;

        let entities: Vec<DatabaseViewMappingEntity> = mappings.into_iter().map(|m| {
            DatabaseViewMappingEntity {
                id: m.id.unwrap().to_hex(),
                name: m.name,
                description: m.description,
                entity_type: m.entity_type,
                database_table_origin_id: m.database_table_origin_id,
                database_table_destiny_id: m.database_table_destiny_id,
                data_view_id: m.data_view_id,
                field_mappings: m.field_mappings,
                status: m.status,
                created_at: m.created_at.to_rfc3339(),
                updated_at: m.updated_at.to_rfc3339(),
            }
        }).collect();

        Ok(entities)
    }

    pub async fn get_database_view_mappings_by_mapping_type(&self, mapping_type: &str) -> AppResult<Vec<DatabaseViewMappingEntity>> {
        let mappings = self.repository.find_by_mapping_type(mapping_type).await?;

        let entities: Vec<DatabaseViewMappingEntity> = mappings.into_iter().map(|m| {
            DatabaseViewMappingEntity {
                id: m.id.unwrap().to_hex(),
                name: m.name,
                description: m.description,
                entity_type: m.entity_type,
                database_table_origin_id: m.database_table_origin_id,
                database_table_destiny_id: m.database_table_destiny_id,
                data_view_id: m.data_view_id,
                field_mappings: m.field_mappings,
                status: m.status,
                created_at: m.created_at.to_rfc3339(),
                updated_at: m.updated_at.to_rfc3339(),
            }
        }).collect();

        Ok(entities)
    }

    pub async fn get_database_view_mappings_by_entity_type(&self, entity_type: &str) -> AppResult<Vec<DatabaseViewMappingEntity>> {
        let mappings = self.repository.find_by_entity_type(entity_type).await?;

        let entities: Vec<DatabaseViewMappingEntity> = mappings.into_iter().map(|m| {
            DatabaseViewMappingEntity {
                id: m.id.unwrap().to_hex(),
                name: m.name,
                description: m.description,
                entity_type: m.entity_type,
                database_table_origin_id: m.database_table_origin_id,
                database_table_destiny_id: m.database_table_destiny_id,
                data_view_id: m.data_view_id,
                field_mappings: m.field_mappings,
                status: m.status,
                created_at: m.created_at.to_rfc3339(),
                updated_at: m.updated_at.to_rfc3339(),
            }
        }).collect();

        Ok(entities)
    }

    pub async fn update_database_view_mapping(&self, id: &str, data: UpdateDatabaseViewMappingDto) -> AppResult<DatabaseViewMappingEntity> {
        let updated = self.repository.update(
            id,
            data.name,
            data.description,
            data.entity_type,
            data.database_table_origin_id,
            data.database_table_destiny_id,
            data.data_view_id,
            data.field_mappings,
            data.status,
        ).await?;

        Ok(DatabaseViewMappingEntity {
            id: updated.id.unwrap().to_hex(),
            name: updated.name,
            description: updated.description,
            entity_type: updated.entity_type,
            database_table_origin_id: updated.database_table_origin_id,
            database_table_destiny_id: updated.database_table_destiny_id,
            data_view_id: updated.data_view_id,
            field_mappings: updated.field_mappings,
            status: updated.status,
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        })
    }

    pub async fn delete_database_view_mapping(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }

    pub async fn generate_fhir_preview(
        &self,
        view_id: &str,
        company_id: &str
    ) -> AppResult<Value> {

        // Fetch database view and mappings together at the start
        let (db_view, db_config, mappings) = if let (Some(view_repo), Some(config_repo)) = 
            (&self.view_repository, &self.config_repository) {
            
            // Get database view
            let view = view_repo.find_by_id(view_id).await?
                .ok_or_else(|| AppError::NotFound("Database view not found".to_string()))?;
            
            // Get database configuration
            let config = config_repo.find_by_id(&view.database_configuration_id).await?
                .ok_or_else(|| AppError::NotFound("Database configuration not found".to_string()))?;
            
            // Get mappings
            let mappings = self.get_database_view_mappings_by_data_view_id(view_id).await?;
            
            (view, config, mappings)
        } else {
            return Err(AppError::Database("Required repositories not available".to_string()));
        };
        
        if mappings.is_empty() {
            return Ok(json!({ "entries": [] }));
        }

        // Collect all transformation IDs (which are database_model owner_ids) from field mappings
        let mut owner_ids: Vec<String> = Vec::new();
        for mapping in &mappings {
            for field_mapping in &mapping.field_mappings {
                if let Some(transformation_id) = &field_mapping.transformation_id {
                    if !transformation_id.is_empty() && !owner_ids.contains(transformation_id) {
                        owner_ids.push(transformation_id.clone());
                    }
                }
            }
        }

        // Fetch all database_model_values by owner_ids
        let mut model_values: std::collections::HashMap<String, DatabaseModelValue> =
             std::collections::HashMap::new();
         if let Some(model_value_repo) = &self.model_value_repository {
             if !owner_ids.is_empty() {
                 let values = model_value_repo.find_by_owner_ids(&owner_ids).await?;
                 for value in values {
                     if let Some(id) = &value.id {
                         model_values.insert(id.to_hex(), value);
                     }
                 }
             }
         }

        // Check if we should create a bundle based on db_view entity_type
        let should_create_bundle = db_view.entity_type == "BUNDLE";
        
        // Generate resources for all mappings with database_model_values
        let mut generated_resources: Vec<Value> = Vec::new();
        
        for mapping in &mappings {
            // Generate resource with model values
            let resource = FhirGenerator::generate_resource_with_model_values(mapping, &model_values, company_id);
            generated_resources.push(resource);
        }
        
        // Build UUID mapping from all generated resources (for bundles)
        let resource_uuids: std::collections::HashMap<String, String> = if should_create_bundle {
            let mut uuids = std::collections::HashMap::new();
            for resource in &generated_resources {
                if let Some(full_url) = resource.get("fullUrl").and_then(|u| u.as_str()) {
                    if let Some(uuid_part) = full_url.strip_prefix("urn:uuid:") {
                        let resource_type = resource.get("resource")
                            .and_then(|r| r.get("resourceType"))
                            .and_then(|rt| rt.as_str())
                            .unwrap_or("");
                        uuids.insert(resource_type.to_string(), uuid_part.to_string());
                    }
                }
            }
            uuids
        } else {
            std::collections::HashMap::new()
        };

        // Try to fetch real data from client database and replace placeholders
        if let Some(table_repo) = &self.table_repository {
            // Build Oracle connection string
            let connection_string = format!(
                "oracle://{}:{}@{}:{}/{}",
                db_config.username,
                db_config.password,
                db_config.host,
                db_config.port,
                db_config.database
            );

            // Connect to client database
            match OracleConnector::new(&connection_string).await {
                Ok(connector) => {
                    // Apply data replacement for each generated resource
                    for (idx, resource) in generated_resources.iter_mut().enumerate() {
                        if idx < mappings.len() {
                            let mapping = &mappings[idx];
                            
                            // Get the origin table for this mapping
                            if let Ok(Some(origin_table)) = table_repo.find_by_id(&mapping.database_table_origin_id).await {
                                let table_name = format!("{}_INTERHEALTH", origin_table.entity_type);
                                
                                // Fetch data for this specific resource
                                match connector.fetch_first_row(&table_name).await {
                                    Ok(data) => {
                                        // Replace placeholders with real data and apply database_model_value transformations
                                        Replacer::replace_in_entry_with_model_values(
                                            resource,
                                            &data,
                                            &mapping.field_mappings,
                                            &model_values,
                                            company_id
                                        );
                                    }
                                    Err(_) => {
                                        println!("Failed to fetch data from table: {}", table_name);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("Failed to connect to client database");
                }
            }
        }
        
        // Convert references to UUIDs AFTER data replacement (for bundles)
        if should_create_bundle && !resource_uuids.is_empty() {
            for resource in &mut generated_resources {
                Self::convert_references_to_uuid(resource, &resource_uuids);
            }
        }
        
        // Create final result - bundle or single resource
        let mut result = if should_create_bundle && generated_resources.len() > 1 {
            // Create a bundle with all generated resources
            json!({
                "resourceType": "Bundle",
                "type": "transaction",
                "entry": generated_resources.into_iter().map(|resource| {
                    json!({
                        "fullUrl": resource.get("fullUrl").cloned().unwrap_or_else(|| json!(format!("urn:uuid:{}", uuid::Uuid::new_v4()))),
                        "resource": resource.get("resource").unwrap_or(&resource).clone(),
                        "request": resource.get("request").cloned().unwrap_or_else(|| {
                            let resource_type = resource.get("resource")
                                .and_then(|r| r.get("resourceType"))
                                .and_then(|rt| rt.as_str())
                                .unwrap_or("Resource");
                            json!({
                                "method": "POST",
                                "url": resource_type
                            })
                        })
                    })
                }).collect::<Vec<Value>>()
            })
        } else if !generated_resources.is_empty() {
            // Single resource
            generated_resources.into_iter().next().unwrap()
        } else {
            json!({ "entries": [] })
        };

        // Validate the FHIR resource and add recommendations
        let validation = Validator::validate(&result);
        
        // Return the resource with validation recommendations
        Ok(json!({
            "resource": result,
            "validation": {
                "isValid": validation.is_valid,
                "recommendations": validation.recommendations
            }
        }))
    }
}
