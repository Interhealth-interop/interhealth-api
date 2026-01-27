use serde_json::{json, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRecommendation {
    pub severity: String,
    pub field: String,
    pub message: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub recommendations: Vec<ValidationRecommendation>,
}

pub struct Validator;

impl Validator {
    /// Validate a FHIR resource and return recommendations
    pub fn validate(resource: &Value) -> ValidationResult {
        let mut recommendations = Vec::new();

        // Check if this is a FHIR entry (has fullUrl, resource, request)
        // If so, validate the nested resource
        if resource.get("fullUrl").is_some() && resource.get("resource").is_some() {
            // This is a FHIR entry, validate the nested resource
            if let Some(nested_resource) = resource.get("resource") {
                Self::validate_resource_or_bundle(nested_resource, &mut recommendations);
            }
        } else {
            // This is a direct resource or bundle
            Self::validate_resource_or_bundle(resource, &mut recommendations);
        }

        let is_valid = !recommendations.iter().any(|r| r.severity == "error");

        ValidationResult {
            is_valid,
            recommendations,
        }
    }

    fn validate_resource_or_bundle(resource: &Value, recommendations: &mut Vec<ValidationRecommendation>) {
        // Check if it's a Bundle or a single resource
        if let Some(resource_type) = resource.get("resourceType").and_then(|v| v.as_str()) {
            if resource_type == "Bundle" {
                Self::validate_bundle(resource, recommendations);
            } else {
                Self::validate_resource(resource, recommendations);
            }
        } else {
            recommendations.push(ValidationRecommendation {
                severity: "error".to_string(),
                field: "resourceType".to_string(),
                message: "Missing required field 'resourceType'".to_string(),
                recommendation: "Every FHIR resource must have a 'resourceType' field".to_string(),
            });
        }
    }

    fn validate_bundle(bundle: &Value, recommendations: &mut Vec<ValidationRecommendation>) {
        // Validate Bundle type
        if let Some(bundle_type) = bundle.get("type").and_then(|v| v.as_str()) {
            let valid_types = ["document", "message", "transaction", "transaction-response", 
                              "batch", "batch-response", "history", "searchset", "collection"];
            if !valid_types.contains(&bundle_type) {
                recommendations.push(ValidationRecommendation {
                    severity: "error".to_string(),
                    field: "type".to_string(),
                    message: format!("Invalid Bundle type: '{}'", bundle_type),
                    recommendation: format!("Bundle type must be one of: {}", valid_types.join(", ")),
                });
            }
        } else {
            recommendations.push(ValidationRecommendation {
                severity: "error".to_string(),
                field: "type".to_string(),
                message: "Missing required field 'type' in Bundle".to_string(),
                recommendation: "Bundle must have a 'type' field (e.g., 'transaction', 'collection')".to_string(),
            });
        }

        // Validate entries
        if let Some(entries) = bundle.get("entry").and_then(|v| v.as_array()) {
            for (idx, entry) in entries.iter().enumerate() {
                // Validate fullUrl
                if entry.get("fullUrl").is_none() {
                    recommendations.push(ValidationRecommendation {
                        severity: "warning".to_string(),
                        field: format!("entry[{}].fullUrl", idx),
                        message: "Missing 'fullUrl' in Bundle entry".to_string(),
                        recommendation: "Consider adding a 'fullUrl' for better resource identification".to_string(),
                    });
                }

                // Validate resource in entry
                if let Some(resource) = entry.get("resource") {
                    Self::validate_resource(resource, recommendations);
                } else {
                    recommendations.push(ValidationRecommendation {
                        severity: "error".to_string(),
                        field: format!("entry[{}].resource", idx),
                        message: "Missing 'resource' in Bundle entry".to_string(),
                        recommendation: "Each Bundle entry must contain a 'resource' object".to_string(),
                    });
                }
            }
        } else {
            recommendations.push(ValidationRecommendation {
                severity: "warning".to_string(),
                field: "entry".to_string(),
                message: "Bundle has no entries".to_string(),
                recommendation: "Consider adding entries to the Bundle".to_string(),
            });
        }
    }

    fn validate_resource(resource: &Value, recommendations: &mut Vec<ValidationRecommendation>) {
        let resource_type = resource.get("resourceType")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        // Validate ID format
        if let Some(id) = resource.get("id").and_then(|v| v.as_str()) {
            if id.is_empty() {
                recommendations.push(ValidationRecommendation {
                    severity: "warning".to_string(),
                    field: "id".to_string(),
                    message: "Resource ID is empty".to_string(),
                    recommendation: "Resource ID should be a non-empty string".to_string(),
                });
            }
        }

        // Validate meta
        if let Some(meta) = resource.get("meta") {
            Self::validate_meta(meta, recommendations);
        }

        // Resource-specific validations
        match resource_type {
            "Patient" => Self::validate_patient(resource, recommendations),
            "Encounter" => Self::validate_encounter(resource, recommendations),
            "Observation" => Self::validate_observation(resource, recommendations),
            "Practitioner" => Self::validate_practitioner(resource, recommendations),
            "Organization" => Self::validate_organization(resource, recommendations),
            "Location" => Self::validate_location(resource, recommendations),
            _ => {}
        }

        // Check for references format
        Self::validate_references(resource, recommendations, "");
    }

    fn validate_meta(meta: &Value, recommendations: &mut Vec<ValidationRecommendation>) {
        // Validate profile URLs
        if let Some(profiles) = meta.get("profile").and_then(|v| v.as_array()) {
            for (idx, profile) in profiles.iter().enumerate() {
                if let Some(url) = profile.as_str() {
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        recommendations.push(ValidationRecommendation {
                            severity: "warning".to_string(),
                            field: format!("meta.profile[{}]", idx),
                            message: "Profile URL should be a valid HTTP(S) URL".to_string(),
                            recommendation: format!("Profile URL '{}' should start with http:// or https://", url),
                        });
                    }
                }
            }
        }
    }

    fn validate_patient(resource: &Value, recommendations: &mut Vec<ValidationRecommendation>) {
        // Check for identifier
        if resource.get("identifier").is_none() {
            recommendations.push(ValidationRecommendation {
                severity: "warning".to_string(),
                field: "identifier".to_string(),
                message: "Patient resource should have at least one identifier".to_string(),
                recommendation: "Add patient identifiers (e.g., MRN, SSN) for better interoperability".to_string(),
            });
        }

        // Check for name
        if resource.get("name").is_none() {
            recommendations.push(ValidationRecommendation {
                severity: "warning".to_string(),
                field: "name".to_string(),
                message: "Patient resource should have a name".to_string(),
                recommendation: "Add patient name for better identification".to_string(),
            });
        }
    }

    fn validate_encounter(resource: &Value, recommendations: &mut Vec<ValidationRecommendation>) {
        // Check for status
        if let Some(status) = resource.get("status").and_then(|v| v.as_str()) {
            let valid_statuses = ["planned", "arrived", "triaged", "in-progress", "onleave", 
                                 "finished", "cancelled", "entered-in-error", "unknown"];
            if !valid_statuses.contains(&status) {
                recommendations.push(ValidationRecommendation {
                    severity: "error".to_string(),
                    field: "status".to_string(),
                    message: format!("Invalid Encounter status: '{}'", status),
                    recommendation: format!("Status must be one of: {}", valid_statuses.join(", ")),
                });
            }
        } else {
            recommendations.push(ValidationRecommendation {
                severity: "error".to_string(),
                field: "status".to_string(),
                message: "Missing required field 'status' in Encounter".to_string(),
                recommendation: "Encounter must have a status (e.g., 'finished', 'in-progress')".to_string(),
            });
        }

        // Check for class
        if resource.get("class").is_none() {
            recommendations.push(ValidationRecommendation {
                severity: "error".to_string(),
                field: "class".to_string(),
                message: "Missing required field 'class' in Encounter".to_string(),
                recommendation: "Encounter must have a class (e.g., 'AMB' for ambulatory, 'IMP' for inpatient)".to_string(),
            });
        }

        // Check for subject reference
        if resource.get("subject").is_none() {
            recommendations.push(ValidationRecommendation {
                severity: "error".to_string(),
                field: "subject".to_string(),
                message: "Missing required field 'subject' in Encounter".to_string(),
                recommendation: "Encounter must reference a Patient in the 'subject' field".to_string(),
            });
        }
    }

    fn validate_observation(resource: &Value, recommendations: &mut Vec<ValidationRecommendation>) {
        // Check for status
        if resource.get("status").is_none() {
            recommendations.push(ValidationRecommendation {
                severity: "error".to_string(),
                field: "status".to_string(),
                message: "Missing required field 'status' in Observation".to_string(),
                recommendation: "Observation must have a status (e.g., 'final', 'preliminary')".to_string(),
            });
        }

        // Check for code
        if resource.get("code").is_none() {
            recommendations.push(ValidationRecommendation {
                severity: "error".to_string(),
                field: "code".to_string(),
                message: "Missing required field 'code' in Observation".to_string(),
                recommendation: "Observation must have a code describing what was observed".to_string(),
            });
        }
    }

    fn validate_practitioner(resource: &Value, recommendations: &mut Vec<ValidationRecommendation>) {
        // Check for identifier or name
        if resource.get("identifier").is_none() && resource.get("name").is_none() {
            recommendations.push(ValidationRecommendation {
                severity: "warning".to_string(),
                field: "identifier/name".to_string(),
                message: "Practitioner should have either an identifier or name".to_string(),
                recommendation: "Add practitioner identifier or name for better identification".to_string(),
            });
        }
    }

    fn validate_organization(resource: &Value, recommendations: &mut Vec<ValidationRecommendation>) {
        // Check for name
        if resource.get("name").is_none() {
            recommendations.push(ValidationRecommendation {
                severity: "warning".to_string(),
                field: "name".to_string(),
                message: "Organization should have a name".to_string(),
                recommendation: "Add organization name for better identification".to_string(),
            });
        }
    }

    fn validate_location(resource: &Value, recommendations: &mut Vec<ValidationRecommendation>) {
        // Check for name or identifier
        if resource.get("name").is_none() && resource.get("identifier").is_none() {
            recommendations.push(ValidationRecommendation {
                severity: "warning".to_string(),
                field: "name/identifier".to_string(),
                message: "Location should have a name or identifier".to_string(),
                recommendation: "Add location name or identifier for better identification".to_string(),
            });
        }
    }

    fn validate_references(value: &Value, recommendations: &mut Vec<ValidationRecommendation>, path: &str) {
        match value {
            Value::Object(map) => {
                // Check if this is a Reference object
                if let Some(reference) = map.get("reference").and_then(|v| v.as_str()) {
                    // Validate reference format (should be ResourceType/id)
                    if !reference.contains('/') && !reference.starts_with('#') && !reference.starts_with("urn:") {
                        let field_path = if path.is_empty() { "reference".to_string() } else { format!("{}.reference", path) };
                        recommendations.push(ValidationRecommendation {
                            severity: "warning".to_string(),
                            field: field_path,
                            message: format!("Reference '{}' may not follow FHIR format", reference),
                            recommendation: "References should follow the format 'ResourceType/id' (e.g., 'Patient/123')".to_string(),
                        });
                    }
                }

                // Recursively check nested objects
                for (key, val) in map {
                    let new_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                    Self::validate_references(val, recommendations, &new_path);
                }
            }
            Value::Array(arr) => {
                for (idx, item) in arr.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, idx);
                    Self::validate_references(item, recommendations, &new_path);
                }
            }
            _ => {}
        }
    }

    /// Create a response with the resource and validation results
    pub fn create_validation_response(resource: Value) -> Value {
        let validation = Self::validate(&resource);
        
        json!({
            "resource": resource,
            "validation": {
                "isValid": validation.is_valid,
                "recommendations": validation.recommendations
            }
        })
    }
}
