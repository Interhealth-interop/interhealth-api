use bson::{doc, Document};

/// Map camelCase field names from API to snake_case field names in database
/// 
/// # Arguments
/// * `field` - Field name from API (potentially in camelCase)
/// 
/// # Returns
/// The corresponding database field name (in snake_case)
fn map_field_name(field: &str) -> String {
    match field {
        // Common field mappings
        "entityType" => "entity_type".to_string(),
        "databaseConfigurationId" => "database_configuration_id".to_string(),
        "companyId" => "company_id".to_string(),
        "userId" => "user_id".to_string(),
        "userType" => "user_type".to_string(),
        "primaryDocument" => "primary_document".to_string(),
        "jobId" => "job_id".to_string(),
        "mainResource" => "main_resource".to_string(),
        "isFhirDestination" => "is_fhir_destination".to_string(),
        "isInterhealthDestination" => "is_interhealth_destination".to_string(),
        "startedAt" => "started_at".to_string(),
        "cancelledAt" => "cancelled_at".to_string(),
        "createdAt" => "created_at".to_string(),
        "updatedAt" => "updated_at".to_string(),
        "dbType" => "db_type".to_string(),
        "authType" => "auth_type".to_string(),
        "ownerId" => "owner_id".to_string(),
        "typeField" => "type_field".to_string(),
        // If no mapping exists, return the field as-is (for already snake_case fields)
        _ => field.to_string(),
    }
}

/// Build a MongoDB sort document from order field and order direction
/// 
/// # Arguments
/// * `order_field` - Optional field name to sort by (in camelCase or snake_case)
/// * `order_by` - Optional sort direction ("ASC" or "DESC")
/// 
/// # Returns
/// A MongoDB sort document, or None if no sorting is specified
pub fn build_sort_document(order_field: Option<String>, order_by: Option<String>) -> Option<Document> {
    if let Some(field) = order_field {
        let direction = match order_by.as_deref() {
            Some("DESC") | Some("desc") => -1,
            _ => 1, // Default to ASC
        };
        
        // Map the field name to database format
        let db_field = map_field_name(&field);
        
        Some(doc! { db_field: direction })
    } else {
        None
    }
}
