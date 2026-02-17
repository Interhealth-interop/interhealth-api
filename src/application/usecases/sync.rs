// Sync use case - orchestrates FHIR transformation for synchronization
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

use crate::domain::entities::{DatabaseViewMapping, DatabaseTransformation};
use crate::infrastructure::repositories::{
    DatabaseViewMappingRepository,
    DatabaseTransformationRepository,
};
use crate::utils::{AppError, AppResult, Replacer};
use crate::application::usecases::fhir::FhirGenerator;
use crate::application::usecases::database_view_mapping::DatabaseViewMappingEntity;

/// Use case for synchronization operations
/// Reuses existing FHIR generation logic from generate_fhir_preview
pub struct SyncUseCase {
    mapping_repo: Arc<DatabaseViewMappingRepository>,
    transformation_repo: Arc<DatabaseTransformationRepository>,
}

impl SyncUseCase {
    pub fn new(
        mapping_repo: Arc<DatabaseViewMappingRepository>,
        transformation_repo: Arc<DatabaseTransformationRepository>,
    ) -> Self {
        Self {
            mapping_repo,
            transformation_repo,
        }
    }

    /// Transform a page of Oracle records to FHIR resources
    /// Returns a list of FHIR resources ready to be sent
    pub async fn transform_records_to_fhir(
        &self,
        view_id: &str,
        records: Vec<HashMap<String, String>>,
    ) -> AppResult<Vec<Value>> {
        
        // Fetch mappings for this view
        let mappings: Vec<DatabaseViewMapping> = self.mapping_repo
            .find_by_data_view_id(view_id)
            .await?;

        if mappings.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch transformations
        let transformations = self.fetch_transformations(&mappings).await?;

        // Generate FHIR resources for each record
        let mut fhir_resources = Vec::new();

        for record in records {
            // For each mapping, generate a FHIR resource
            for mapping in &mappings {
                let resource = self.generate_fhir_resource(
                    mapping,
                    &transformations,
                    &record,
                ).await?;
                
                fhir_resources.push(resource);
            }
        }

        Ok(fhir_resources)
    }

    /// Convert DatabaseViewMapping (domain entity) to DatabaseViewMappingEntity (DTO)
    fn to_entity(&self, mapping: &DatabaseViewMapping) -> DatabaseViewMappingEntity {
        DatabaseViewMappingEntity {
            id: mapping.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
            name: mapping.name.clone(),
            description: mapping.description.clone(),
            entity_type: mapping.entity_type.clone(),
            resource: mapping.resource.clone(),
            database_table_origin_id: mapping.database_table_origin_id.clone(),
            database_table_destiny_id: mapping.database_table_destiny_id.clone(),
            data_view_id: mapping.data_view_id.clone(),
            field_mappings: mapping.field_mappings.clone(),
            status: mapping.status.clone(),
            created_at: mapping.created_at.to_rfc3339(),
            updated_at: mapping.updated_at.to_rfc3339(),
        }
    }

    /// Generate a single FHIR resource from Oracle data
    async fn generate_fhir_resource(
        &self,
        mapping: &DatabaseViewMapping,
        transformations: &HashMap<String, DatabaseTransformation>,
        record: &HashMap<String, String>,
    ) -> AppResult<Value> {
        
        // Convert to entity type (DTO)
        let mapping_entity = self.to_entity(mapping);
        
        // Filter transformations for this mapping
        let mapping_transformations: HashMap<String, DatabaseTransformation> = 
            transformations.iter()
                .filter(|(id, _)| {
                    mapping.field_mappings.iter().any(|fm| {
                        fm.transformation_id.as_ref().map_or(false, |tid| tid == *id)
                    })
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
        
        // Generate FHIR template with placeholders
        let mut resource = FhirGenerator::generate_resource_with_transformations(
            &mapping_entity,
            &mapping_transformations,
        );

        // Replace placeholders with actual data and apply transformations
        Replacer::replace_in_entry_with_transformations(
            &mut resource,
            record,
            &mapping.field_mappings,
            &mapping_transformations,
        );

        // Extract the resource if it's wrapped in an entry
        if let Some(extracted_resource) = resource.get("resource") {
            Ok(extracted_resource.clone())
        } else {
            Ok(resource)
        }
    }

    /// Fetch all transformations needed for the mappings
    async fn fetch_transformations(
        &self,
        mappings: &[DatabaseViewMapping],
    ) -> AppResult<HashMap<String, DatabaseTransformation>> {
        
        // Collect unique transformation IDs
        let mut transformation_ids = Vec::new();
        for mapping in mappings {
            for field_mapping in &mapping.field_mappings {
                if let Some(transformation_id) = &field_mapping.transformation_id {
                    if !transformation_id.is_empty() && !transformation_ids.contains(transformation_id) {
                        transformation_ids.push(transformation_id.clone());
                    }
                }
            }
        }

        // Fetch transformations from database
        let mut transformations = HashMap::new();
        for transformation_id in transformation_ids {
            if let Ok(Some(transformation)) = self.transformation_repo.find_by_id(&transformation_id).await {
                transformations.insert(transformation_id.clone(), transformation);
            }
        }

        Ok(transformations)
    }
}
