use std::collections::HashMap;
use serde_json::Value;
use bson::oid::ObjectId;
use crate::domain::entities::{FieldMapping, DatabaseTransformation, DatabaseModelValue};

/// Utility for replacing FHIR placeholder values with real database values
pub struct Replacer;

impl Replacer {
    /// Replace placeholder column names with real values in a FHIR resource
    /// Example: "patient_status" -> "active"
    pub fn replace_in_resource(
        resource: &mut Value,
        data: &HashMap<String, String>,
    ) {
        match resource {
            Value::Object(map) => {
                for (_, value) in map.iter_mut() {
                    Self::replace_in_resource(value, data);
                }
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    Self::replace_in_resource(item, data);
                }
            }
            Value::String(s) => {
                // Check if this string matches a column name (case-insensitive)
                let lower = s.to_lowercase();
                
                // First, try direct match
                if let Some(real_value) = data.get(&lower) {
                    *s = real_value.clone();
                } else if s.contains('/') {
                    // Handle prefixed references like "Patient/encounter_patient_code"
                    // Split by '/' and check if the second part is a column name
                    let parts: Vec<&str> = s.split('/').collect();
                    if parts.len() == 2 {
                        let prefix = parts[0];
                        let column_name = parts[1].to_lowercase();
                        
                        if let Some(real_value) = data.get(&column_name) {
                            // Reconstruct with prefix and real value
                            *s = format!("{}/{}", prefix, real_value);
                        }
                    }
                }
            }
            Value::Null => {
                // Replace null values with empty strings
                *resource = Value::String(String::new());
            }
            _ => {}
        }
    }

    /// Replace placeholders in a FHIR bundle (multiple entries)
    pub fn replace_in_bundle(
        bundle: &mut Value,
        data: &HashMap<String, String>,
    ) {
        if let Some(entries) = bundle.get_mut("entry").and_then(|e| e.as_array_mut()) {
            for entry in entries.iter_mut() {
                if let Some(resource) = entry.get_mut("resource") {
                    Self::replace_in_resource(resource, data);
                }
            }
        }
    }

    /// Replace placeholders with transformations applied
    pub fn replace_in_bundle_with_transformations(
        bundle: &mut Value,
        data: &HashMap<String, String>,
        field_mappings: &[FieldMapping],
        transformations: &HashMap<String, DatabaseTransformation>,
    ) {
        // Apply transformations to data
        let transformed_data = Self::apply_transformations(data, field_mappings, transformations);
        
        // Replace with transformed data
        Self::replace_in_bundle(bundle, &transformed_data);
    }

    /// Replace placeholders in a single FHIR entry (resource + request)
    pub fn replace_in_entry(
        entry: &mut Value,
        data: &HashMap<String, String>,
    ) {
        if let Some(resource) = entry.get_mut("resource") {
            Self::replace_in_resource(resource, data);
        }
        
        // Also replace in request.ifNoneExist if present
        if let Some(request) = entry.get_mut("request") {
            if let Some(if_none_exist) = request.get_mut("ifNoneExist") {
                if let Value::String(s) = if_none_exist {
                    // The ifNoneExist should already be in format: "identifier=system|value"
                    // We just need to replace any placeholder column names that might be in the value part
                    for (col_name, real_value) in data {
                        *s = s.replace(col_name, real_value);
                    }
                }
            }
        }
    }

    /// Replace placeholders in a single entry with transformations applied
    pub fn replace_in_entry_with_transformations(
        entry: &mut Value,
        data: &HashMap<String, String>,
        field_mappings: &[FieldMapping],
        transformations: &HashMap<String, DatabaseTransformation>,
    ) {
        // Apply transformations to data
        let transformed_data = Self::apply_transformations(data, field_mappings, transformations);
        
        // Replace with transformed data
        Self::replace_in_entry(entry, &transformed_data);
        
        // Add display attributes for fields that have transformations and end with .code
        if let Some(resource) = entry.get_mut("resource") {
            Self::add_display_attributes(resource, data, field_mappings, transformations);
        }
    }

    /// Replace placeholders in a single entry with database_model_value transformations applied
    pub fn replace_in_entry_with_model_values(
        entry: &mut Value,
        data: &HashMap<String, String>,
        field_mappings: &[FieldMapping],
        model_values: &HashMap<String, DatabaseModelValue>,
        company_id: &str,
    ) {
        // Parse company_id to ObjectId for comparison
        let company_object_id = ObjectId::parse_str(company_id).ok();
        
        // Apply database_model_value transformations to data
        let transformed_data = Self::apply_model_value_transformations(
            data, 
            field_mappings, 
            model_values, 
            company_object_id.as_ref()
        );
        
        // Replace with transformed data
        Self::replace_in_entry(entry, &transformed_data);
        
        // Add display attributes for fields that have transformations and end with .code
        if let Some(resource) = entry.get_mut("resource") {
            Self::add_model_value_display_attributes(
                resource, 
                data, 
                field_mappings, 
                model_values, 
                company_object_id.as_ref()
            );
        }
    }

    /// Apply transformations to data values based on field mappings
    fn apply_transformations(
        data: &HashMap<String, String>,
        field_mappings: &[FieldMapping],
        transformations: &HashMap<String, DatabaseTransformation>,
    ) -> HashMap<String, String> {
        let mut transformed_data = data.clone();
        
        // For each field mapping with a transformation
        for field_mapping in field_mappings {
            if let Some(transformation_id) = &field_mapping.transformation_id {
                if let Some(transformation) = transformations.get(transformation_id) {
                    // Get the column name (field_origin)
                    let column_name = field_mapping.field_origin.to_lowercase();
                    
                    // Get the actual value from data
                    if let Some(original_value) = data.get(&column_name) {
                        // Look up the transformed value
                        if let Some(mapped_value) = transformation.value_mappings.get(original_value) {
                            // Replace with transformed value
                            transformed_data.insert(column_name, mapped_value.code.clone());
                        }
                    }
                }
            }
        }
        
        transformed_data
    }

    /// Apply database_model_value transformations to data values based on field mappings
    fn apply_model_value_transformations(
        data: &HashMap<String, String>,
        field_mappings: &[FieldMapping],
        model_values: &HashMap<String, DatabaseModelValue>,
        company_object_id: Option<&ObjectId>,
    ) -> HashMap<String, String> {
        let mut transformed_data = data.clone();
        
        // For each field mapping with a transformation_id
        for field_mapping in field_mappings {
            if let Some(transformation_id) = &field_mapping.transformation_id {
                // Get the column name (field_origin)
                let column_name = field_mapping.field_origin.to_lowercase();
                
                // Get the actual value from data (e.g., "F")
                if let Some(original_value) = data.get(&column_name) {
                    // transformation_id is the database_model owner_id
                    // Find all database_model_values with matching owner_id
                    for model_value in model_values.values() {
                        if model_value.owner_id.to_hex() == *transformation_id {
                            // Look for company-specific client mapping with matching source_key
                            if let Some(company_oid) = company_object_id {
                                if let Some(_client_mapping) = model_value.clients.iter()
                                    .find(|c| c.company_id == *company_oid && c.source_key == *original_value) {
                                    // Transform: source_key ("F") -> code ("female")
                                    transformed_data.insert(column_name.clone(), model_value.code.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        transformed_data
    }

    /// Add display attributes to code fields that have database_model_value transformations
    fn add_model_value_display_attributes(
        resource: &mut Value,
        data: &HashMap<String, String>,
        field_mappings: &[FieldMapping],
        model_values: &HashMap<String, DatabaseModelValue>,
        company_object_id: Option<&ObjectId>,
    ) {
        // For each field mapping with a transformation that ends with .code
        for field_mapping in field_mappings {
            if let Some(transformation_id) = &field_mapping.transformation_id {
                if field_mapping.field_destiny.ends_with(".code") {
                    // Get the column name (field_origin)
                    let column_name = field_mapping.field_origin.to_lowercase();
                    
                    // Get the actual value from data (e.g., "F")
                    if let Some(original_value) = data.get(&column_name) {
                        // transformation_id is the database_model owner_id
                        // Find database_model_value with matching owner_id
                        for model_value in model_values.values() {
                            if model_value.owner_id.to_hex() == *transformation_id {
                                // Look for company-specific client mapping
                                if let Some(company_oid) = company_object_id {
                                    if let Some(_client_mapping) = model_value.clients.iter()
                                        .find(|c| c.company_id == *company_oid && c.source_key == *original_value) {
                                        // Create the display path by replacing .code with .display
                                        let display_path = field_mapping.field_destiny.replace(".code", ".display");
                                        
                                        // Set the display value from database_model_value description
                                        Self::set_value_by_path(resource, &display_path, &model_value.description);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Add display attributes to code fields that have transformations
    fn add_display_attributes(
        resource: &mut Value,
        data: &HashMap<String, String>,
        field_mappings: &[FieldMapping],
        transformations: &HashMap<String, DatabaseTransformation>,
    ) {
        // For each field mapping with a transformation that ends with .code
        for field_mapping in field_mappings {
            if let Some(transformation_id) = &field_mapping.transformation_id {
                if field_mapping.field_destiny.ends_with(".code") {
                    if let Some(transformation) = transformations.get(transformation_id) {
                        // Get the column name (field_origin)
                        let column_name = field_mapping.field_origin.to_lowercase();
                        
                        // Get the actual value from data
                        if let Some(original_value) = data.get(&column_name) {
                            // Look up the transformed value to get the description
                            if let Some(mapped_value) = transformation.value_mappings.get(original_value) {
                                // Create the display path by replacing .code with .display
                                let display_path = field_mapping.field_destiny.replace(".code", ".display");
                                
                                // Convert path to JSON pointer format and set the value
                                // e.g., "extension[2].valueCodeableConcept.coding[0].display" 
                                // becomes "/extension/2/valueCodeableConcept/coding/0/display"
                                let _pointer_path = Self::dot_notation_to_pointer(&display_path);
                                
                                // Navigate to the parent and set the display field
                                Self::set_value_by_path(resource, &display_path, &mapped_value.description);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Convert dot notation path to JSON pointer format
    fn dot_notation_to_pointer(path: &str) -> String {
        path.replace(".", "/")
            .replace("[", "/")
            .replace("]", "")
    }

    /// Set a value in nested JSON structure using dot notation path
    fn set_value_by_path(obj: &mut Value, path: &str, value: &str) {
        let parts: Vec<String> = path.split('.').map(|s| s.to_string()).collect();
        Self::set_value_recursive(obj, &parts, 0, value);
    }

    /// Recursively set a value in nested JSON structure
    fn set_value_recursive(obj: &mut Value, parts: &[String], index: usize, value: &str) {
        if index >= parts.len() {
            return;
        }

        let part = &parts[index];
        let is_last = index == parts.len() - 1;

        if part.contains('[') {
            // Handle array notation
            let bracket_pos = part.find('[').unwrap();
            let field = part[..bracket_pos].to_string();
            let index_str = &part[bracket_pos + 1..part.len() - 1];
            let arr_index: usize = index_str.parse().unwrap_or(0);

            // Ensure field exists as array
            if obj[&field].is_null() {
                obj[field.clone()] = Value::Array(vec![]);
            }

            if let Some(arr) = obj[&field].as_array_mut() {
                // Ensure array has enough elements
                while arr.len() <= arr_index {
                    arr.push(Value::Object(serde_json::Map::new()));
                }

                if is_last {
                    arr[arr_index] = Value::String(value.to_string());
                } else {
                    Self::set_value_recursive(&mut arr[arr_index], parts, index + 1, value);
                }
            }
        } else {
            // Regular field
            if is_last {
                obj[part.as_str()] = Value::String(value.to_string());
            } else {
                // Ensure field exists
                if obj[part.as_str()].is_null() {
                    obj[part.clone()] = Value::Object(serde_json::Map::new());
                }
                Self::set_value_recursive(&mut obj[part.as_str()], parts, index + 1, value);
            }
        }
    }
}