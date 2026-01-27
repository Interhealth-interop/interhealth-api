use serde_json::{json, Value};
use super::database_view_mapping::DatabaseViewMappingEntity;
use crate::domain::fhir::r4::{bundle, resource};
use crate::domain::entities::DatabaseTransformation;
use std::collections::HashMap;
use uuid::Uuid;

pub struct FhirGenerator;

impl FhirGenerator {
    pub fn generate_bundle(mappings: &[DatabaseViewMappingEntity]) -> Value {
        Self::generate_bundle_with_transformations(mappings, &HashMap::new())
    }

    pub fn generate_bundle_with_transformations(
        mappings: &[DatabaseViewMappingEntity],
        transformations: &HashMap<String, DatabaseTransformation>
    ) -> Value {
        let mut bundle = bundle::get_template();
        let mut entries = Vec::new();

        for mapping in mappings {
            let resource_entry = Self::generate_resource_with_transformations(mapping, transformations);
            entries.push(resource_entry);
        }

        if let Some(bundle_obj) = bundle.as_object_mut() {
            bundle_obj.insert("entry".to_string(), json!(entries));
        }

        bundle
    }

    pub fn generate_resource(mapping: &DatabaseViewMappingEntity) -> Value {
        Self::generate_resource_with_transformations(mapping, &HashMap::new())
    }

    pub fn generate_resource_with_transformations(
        mapping: &DatabaseViewMappingEntity,
        transformations: &HashMap<String, DatabaseTransformation>
    ) -> Value {
        let mut resource_template = resource::get_template();
        
        if let Some(resource_obj) = resource_template.as_object_mut() {
            let entity_type = &mapping.entity_type;
            let resource_type = Self::map_entity_type_to_fhir_resource(entity_type);
            
            // Generate UUID for fullUrl
            let uuid = Uuid::new_v4();
            resource_obj.insert("fullUrl".to_string(), json!(format!("urn:uuid:{}", uuid)));
            
            if let Some(resource) = resource_obj.get_mut("resource").and_then(|r| r.as_object_mut()) {
                resource.insert("resourceType".to_string(), json!(resource_type));
                
                // Update meta tags with actual values
                if let Some(meta) = resource.get_mut("meta").and_then(|m| m.as_object_mut()) {
                    if let Some(tags) = meta.get_mut("tag").and_then(|t| t.as_array_mut()) {
                        // Update client-id tag
                        if let Some(tag) = tags.get_mut(0).and_then(|t| t.as_object_mut()) {
                            tag.insert("code".to_string(), json!("INTERHEALTH"));
                        }
                        // Update data-provider tag
                        if let Some(tag) = tags.get_mut(1).and_then(|t| t.as_object_mut()) {
                            tag.insert("code".to_string(), json!("interhealth"));
                        }
                        // Update data-type tag
                        if let Some(tag) = tags.get_mut(2).and_then(|t| t.as_object_mut()) {
                            tag.insert("code".to_string(), json!(format!("{}-Resource", resource_type)));
                        }
                    }
                }
                
                let mut resource_data = json!({});
                if let Some(data_obj) = resource_data.as_object_mut() {
                    for field_mapping in &mapping.field_mappings {
                        // Process field even if origin is empty, but only if referenceDestiny exists
                        let has_value = !field_mapping.field_origin.is_empty();
                        let has_reference = field_mapping.reference_destiny.is_some();
                        
                        if has_value || has_reference {
                            let mut origin_value = if has_value {
                                format!("{}", field_mapping.field_origin)
                            } else {
                                String::new()
                            };
                            let mut display_value: Option<String> = None;
                            
                            // Apply transformation if transformation_id exists and we have a value
                            if has_value {
                                if let Some(transformation_id) = &field_mapping.transformation_id {
                                    if let Some(transformation) = transformations.get(transformation_id) {
                                        if let Some(mapping_value) = transformation.value_mappings.get(&origin_value) {
                                            // Extract code and description from transformation
                                            origin_value = mapping_value.code.clone();
                                            display_value = Some(mapping_value.description.clone());
                                        }
                                    }
                                }
                            }
                            
                            // If relationshipDestiny exists and field ends with ".reference", prefix the value
                            if let Some(relationship) = &field_mapping.relationship_destiny {
                                if field_mapping.field_destiny.ends_with(".reference") && has_value {
                                    origin_value = format!("{}/{}", relationship, origin_value);
                                }
                            }
                            
                            // If referenceDestiny exists, merge it with the field value
                            if let Some(reference_destiny) = &field_mapping.reference_destiny {
                                Self::set_nested_value_with_reference(data_obj, &field_mapping.field_destiny, json!(origin_value), reference_destiny);
                                
                                // If field ends with ".code" and we have a display value, add the display field
                                if field_mapping.field_destiny.ends_with(".code") {
                                    if let Some(display) = display_value {
                                        // Replace ".code" with ".display" in the path
                                        let display_path = field_mapping.field_destiny.replace(".code", ".display");
                                        Self::set_nested_value_with_reference(data_obj, &display_path, json!(display), reference_destiny);
                                    }
                                }
                            } else if has_value {
                                Self::set_nested_value(data_obj, &field_mapping.field_destiny, json!(origin_value));
                                
                                // If field ends with ".code" and we have a display value, add the display field
                                if field_mapping.field_destiny.ends_with(".code") {
                                    if let Some(display) = display_value {
                                        let display_path = field_mapping.field_destiny.replace(".code", ".display");
                                        Self::set_nested_value(data_obj, &display_path, json!(display));
                                    }
                                }
                            }
                        }
                    }
                }
                
                if let Some(data_obj) = resource_data.as_object() {
                    for (key, value) in data_obj {
                        resource.insert(key.clone(), value.clone());
                    }
                }
            }
            
            // Build ifNoneExist before borrowing resource_obj mutably
            let if_none_exist = Self::build_if_none_exist_from_resource(resource_obj);
            
            if let Some(request) = resource_obj.get_mut("request").and_then(|r| r.as_object_mut()) {
                request.insert("method".to_string(), json!("POST"));
                request.insert("url".to_string(), json!(resource_type));
                request.insert("ifNoneExist".to_string(), json!(if_none_exist));
            }
        }
        
        resource_template
    }

    fn map_entity_type_to_fhir_resource(entity_type: &str) -> String {
        let lower = entity_type.to_lowercase();
        let mut chars = lower.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// Build ifNoneExist query string from the first identifier in the resource
    /// Format: "system|value" (e.g., "http://interop.interhealth.com.br/NamingSystem/ns-codigo|000000177482")
    fn build_if_none_exist_from_resource(resource_obj: &serde_json::Map<String, Value>) -> String {
        // Look for identifier array in the resource
        if let Some(resource) = resource_obj.get("resource").and_then(|r| r.as_object()) {
            if let Some(identifiers) = resource.get("identifier").and_then(|i| i.as_array()) {
                // Get the first identifier
                if let Some(first_identifier) = identifiers.get(0).and_then(|i| i.as_object()) {
                    let system = first_identifier.get("system")
                        .and_then(|s| s.as_str())
                        .unwrap_or("__IDENTIFIER_SYSTEM__");
                    let value = first_identifier.get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("__IDENTIFIER_VALUE__");
                    
                    return format!("identifier={}|{}", system, value);
                }
            }
        }
        
        // Fallback if no identifier found - use placeholders
        "identifier=__IDENTIFIER_SYSTEM__|__IDENTIFIER_VALUE__".to_string()
    }

    fn set_nested_value_with_reference(
        obj: &mut serde_json::Map<String, Value>,
        path: &str,
        value: Value,
        reference_destiny: &std::collections::HashMap<String, String>
    ) {
        let parts: Vec<&str> = path.split('.').collect();
        
        // Handle simple field (no dots, no arrays)
        if parts.len() == 1 && !path.contains('[') {
            obj.insert(path.to_string(), value);
            return;
        }
        
        let first = parts[0];
        
        // Handle array notation in first part
        if first.contains('[') && first.contains(']') {
            let array_path = Self::parse_array_path(first);
            if let Some((field, indices)) = array_path {
                if !obj.contains_key(&field) {
                    obj.insert(field.clone(), json!([]));
                }
                
                if let Some(array) = obj.get_mut(&field).and_then(|v| v.as_array_mut()) {
                    for &idx in &indices {
                        while array.len() <= idx {
                            array.push(json!({}));
                        }
                        
                        if parts.len() > 1 {
                            // There's more path to go (e.g., "extension[0].valueCodeableConcept.coding[0].code")
                            let rest = parts[1..].join(".");
                            
                            if let Some(nested_obj) = array[idx].as_object_mut() {
                                // Process reference fields for this array element
                                // Match fields that start with "field[idx]." and strip that prefix
                                let mut nested_references = HashMap::new();
                                let current_path = format!("{}.", first); // e.g., "extension[2]."
                                
                                for (key, val) in reference_destiny {
                                    if key.starts_with(&current_path) {
                                        // This reference is for this array element or deeper
                                        // Strip the "field[idx]." prefix
                                        let remaining = key.strip_prefix(&current_path).unwrap();
                                        
                                        if !remaining.contains('.') && !remaining.contains('[') {
                                            // Simple field at this level (e.g., "url", "system", "use")
                                            nested_obj.insert(remaining.to_string(), json!(val));
                                        } else {
                                            // Nested path - pass it down
                                            nested_references.insert(remaining.to_string(), val.clone());
                                        }
                                    }
                                }
                                
                                // Recursively handle the rest of the path with nested references
                                Self::set_nested_value_with_reference(nested_obj, &rest, value.clone(), &nested_references);
                            }
                        } else {
                            // No more path, just set the value
                            array[idx] = value.clone();
                        }
                    }
                }
            }
        } else {
            // Regular nested object (no array in first part)
            if !obj.contains_key(first) {
                obj.insert(first.to_string(), json!({}));
            }
            
            if let Some(nested_obj) = obj.get_mut(first).and_then(|v| v.as_object_mut()) {
                if parts.len() > 1 {
                    let rest = parts[1..].join(".");
                    
                    // Filter reference_destiny for fields that match the current path
                    // For path "valueCodeableConcept" and key "valueCodeableConcept.coding[0].system"
                    // We want to pass down "coding[0].system"
                    let mut nested_references = HashMap::new();
                    let prefix_dot = format!("{}.", first);
                    let prefix_bracket = format!("{}[", first);
                    
                    for (key, val) in reference_destiny {
                        if key.starts_with(&prefix_dot) {
                            // Strip "field." prefix
                            let remaining = key.strip_prefix(&prefix_dot).unwrap();
                            nested_references.insert(remaining.to_string(), val.clone());
                        } else if key.starts_with(&prefix_bracket) {
                            // Strip "field" prefix, keep "[...]"
                            let remaining = key.strip_prefix(first).unwrap();
                            nested_references.insert(remaining.to_string(), val.clone());
                        }
                    }
                    
                    // Continue recursively building the structure
                    Self::set_nested_value_with_reference(nested_obj, &rest, value, &nested_references);
                } else {
                    // This is the final field
                    nested_obj.insert(first.to_string(), value);
                }
            }
        }
    }

    fn set_nested_value(obj: &mut serde_json::Map<String, Value>, path: &str, value: Value) {
        let parts: Vec<&str> = path.split('.').collect();
        
        if parts.len() == 1 {
            if path.contains('[') && path.contains(']') {
                let array_path = Self::parse_array_path(path);
                if let Some((field, indices)) = array_path {
                    Self::ensure_nested_array(obj, &field, &indices, value);
                }
            } else {
                obj.insert(path.to_string(), value);
            }
            return;
        }
        
        let first = parts[0];
        let rest = parts[1..].join(".");
        
        if first.contains('[') && first.contains(']') {
            let array_path = Self::parse_array_path(first);
            if let Some((field, indices)) = array_path {
                if !obj.contains_key(&field) {
                    obj.insert(field.clone(), json!([]));
                }
                
                if let Some(array) = obj.get_mut(&field).and_then(|v| v.as_array_mut()) {
                    for &idx in &indices {
                        while array.len() <= idx {
                            array.push(json!({}));
                        }
                        
                        if let Some(nested_obj) = array[idx].as_object_mut() {
                            Self::set_nested_value(nested_obj, &rest, value.clone());
                        }
                    }
                }
            }
        } else {
            if !obj.contains_key(first) {
                obj.insert(first.to_string(), json!({}));
            }
            
            if let Some(nested_obj) = obj.get_mut(first).and_then(|v| v.as_object_mut()) {
                Self::set_nested_value(nested_obj, &rest, value);
            }
        }
    }

    fn parse_array_path(path: &str) -> Option<(String, Vec<usize>)> {
        let mut field = String::new();
        let mut indices = Vec::new();
        let mut current_index = String::new();
        let mut in_bracket = false;
        
        for ch in path.chars() {
            match ch {
                '[' => {
                    in_bracket = true;
                    current_index.clear();
                }
                ']' => {
                    in_bracket = false;
                    if let Ok(idx) = current_index.parse::<usize>() {
                        indices.push(idx);
                    }
                }
                _ => {
                    if in_bracket {
                        current_index.push(ch);
                    } else {
                        field.push(ch);
                    }
                }
            }
        }
        
        if !field.is_empty() {
            Some((field, indices))
        } else {
            None
        }
    }

    fn ensure_nested_array(obj: &mut serde_json::Map<String, Value>, field: &str, indices: &[usize], value: Value) {
        if !obj.contains_key(field) {
            obj.insert(field.to_string(), json!([]));
        }
        
        if let Some(array) = obj.get_mut(field).and_then(|v| v.as_array_mut()) {
            for &idx in indices {
                while array.len() <= idx {
                    array.push(json!(null));
                }
                array[idx] = value.clone();
            }
        }
    }

    fn set_reference_metadata(obj: &mut serde_json::Map<String, Value>, path: &str, reference: &std::collections::HashMap<String, String>) {
        // Extract the parent path (everything before the last field)
        // For example: "telecom[0].value" -> "telecom[0]"
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return;
        }
        
        // Get parent path (all parts except the last one)
        let parent_path = if parts.len() > 1 {
            parts[..parts.len() - 1].join(".")
        } else {
            // If there's no parent (single field), we can't add reference metadata
            return;
        };
        
        // Navigate to the parent object and add reference fields
        let parent_parts: Vec<&str> = parent_path.split('.').collect();
        Self::navigate_and_set_reference(obj, &parent_parts, reference);
    }

    fn navigate_and_set_reference(obj: &mut serde_json::Map<String, Value>, parts: &[&str], reference: &std::collections::HashMap<String, String>) {
        if parts.is_empty() {
            // We're at the target, add reference fields
            for (key, value) in reference {
                obj.insert(key.clone(), json!(value));
            }
            return;
        }
        
        let first = parts[0];
        let rest = &parts[1..];
        
        if first.contains('[') && first.contains(']') {
            let array_path = Self::parse_array_path(first);
            if let Some((field, indices)) = array_path {
                if !obj.contains_key(&field) {
                    obj.insert(field.clone(), json!([]));
                }
                
                if let Some(array) = obj.get_mut(&field).and_then(|v| v.as_array_mut()) {
                    for &idx in &indices {
                        while array.len() <= idx {
                            array.push(json!({}));
                        }
                        
                        if let Some(nested_obj) = array[idx].as_object_mut() {
                            Self::navigate_and_set_reference(nested_obj, rest, reference);
                        }
                    }
                }
            }
        } else {
            if !obj.contains_key(first) {
                obj.insert(first.to_string(), json!({}));
            }
            
            if let Some(nested_obj) = obj.get_mut(first).and_then(|v| v.as_object_mut()) {
                Self::navigate_and_set_reference(nested_obj, rest, reference);
            }
        }
    }
}