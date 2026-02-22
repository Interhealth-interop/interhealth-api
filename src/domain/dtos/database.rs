use serde::{Deserialize, Serialize};
use validator::Validate;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::de::Error as DeError;

pub use crate::domain::entities::ValueMappingItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseDto {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryDto {
    pub statement: String,
    pub binds: Vec<FilterDto>,
    pub opts: serde_json::Value,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterDto {
    pub field: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDto {
    pub rows: serde_json::Value,
    pub total: Option<i32>,
    pub limit: Option<i32>,
    pub pages: Option<i32>,
    pub page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDatabaseConfigurationDto {
    pub name: String,
    #[serde(rename = "type")]
    pub db_type: String,
    pub version: Option<String>,
    pub host: String,
    pub port: Option<i32>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(rename = "authType")]
    pub auth_type: Option<String>,
    pub credentials: Option<String>,
    pub company_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatabaseConfigurationDto {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub db_type: Option<String>,
    pub version: Option<String>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(rename = "authType")]
    pub auth_type: Option<String>,
    pub credentials: Option<String>,
    pub company_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfigurationEntity {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub db_type: String,
    pub version: Option<String>,
    pub host: String,
    pub port: Option<i32>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(rename = "authType")]
    pub auth_type: Option<String>,
    pub credentials: Option<String>,
    pub company_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTargetIntegrationDto {
    pub name: String,
    pub version: Option<String>,
    pub host: String,
    #[serde(rename = "authType")]
    pub auth_type: Option<String>,
    pub credentials: Option<String>,
    #[serde(rename = "databaseViewId")]
    pub database_view_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTargetIntegrationDto {
    pub name: Option<String>,
    pub version: Option<String>,
    pub host: Option<String>,
    #[serde(rename = "authType")]
    pub auth_type: Option<String>,
    pub credentials: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetIntegrationEntity {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub host: String,
    #[serde(rename = "authType")]
    pub auth_type: Option<String>,
    pub credentials: Option<String>,
    #[serde(rename = "databaseViewId")]
    pub database_view_id: String,
    pub company_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIntegrationControlDto {
    pub name: String,
    #[serde(rename = "databaseViewId")]
    pub database_view_id: String,
    pub cron: String,
    #[serde(rename = "dateField")]
    pub date_field: String,
    #[serde(
        rename = "startAt",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_datetime"
    )]
    pub start_at: Option<DateTime<Utc>>,
    #[serde(
        rename = "endAt",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_datetime"
    )]
    pub end_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIntegrationControlDto {
    pub name: Option<String>,
    pub cron: Option<String>,
    #[serde(rename = "dateField")]
    pub date_field: Option<String>,
    #[serde(
        rename = "startAt",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_datetime"
    )]
    pub start_at: Option<DateTime<Utc>>,
    #[serde(
        rename = "endAt",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_datetime"
    )]
    pub end_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
}

fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;

    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        serde_json::Value::String(s) => DateTime::parse_from_rfc3339(&s)
            .map(|dt| Some(dt.with_timezone(&Utc)))
            .map_err(|_| D::Error::custom("Invalid datetime format")),
        serde_json::Value::Object(map) => {
            if let Some(date_value) = map.get("$date") {
                match date_value {
                    serde_json::Value::String(s) => DateTime::parse_from_rfc3339(s)
                        .map(|dt| Some(dt.with_timezone(&Utc)))
                        .map_err(|_| D::Error::custom("Invalid datetime format")),
                    serde_json::Value::Number(n) => {
                        let ms = n
                            .as_i64()
                            .ok_or_else(|| D::Error::custom("Invalid datetime format"))?;
                        Ok(Some(DateTime::<Utc>::from_timestamp_millis(ms).ok_or_else(|| {
                            D::Error::custom("Invalid datetime format")
                        })?))
                    }
                    serde_json::Value::Object(inner) => {
                        if let Some(serde_json::Value::String(num)) = inner.get("$numberLong") {
                            let ms = num
                                .parse::<i64>()
                                .map_err(|_| D::Error::custom("Invalid datetime format"))?;
                            Ok(Some(DateTime::<Utc>::from_timestamp_millis(ms).ok_or_else(|| {
                                D::Error::custom("Invalid datetime format")
                            })?))
                        } else {
                            Err(D::Error::custom("Invalid datetime format"))
                        }
                    }
                    _ => Err(D::Error::custom("Invalid datetime format")),
                }
            } else {
                Err(D::Error::custom("Invalid datetime format"))
            }
        }
        _ => Err(D::Error::custom("Invalid datetime format")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationControlEntity {
    pub id: String,
    pub name: String,
    #[serde(rename = "databaseViewId")]
    pub database_view_id: String,
    pub cron: String,
    #[serde(rename = "dateField")]
    pub date_field: String,
    #[serde(rename = "startAt", default, skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    #[serde(rename = "endAt", default, skip_serializing_if = "Option::is_none")]
    pub end_at: Option<String>,
    #[serde(rename = "lastRunAt", default, skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<String>,
    pub status: String,
    pub company_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatabaseColumnDto {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "dataType")]
    pub data_type: String,
    #[serde(rename = "isNullable")]
    pub is_nullable: bool,
    #[serde(rename = "isPrimaryKey")]
    pub is_primary_key: bool,
    #[serde(rename = "isForeignKey")]
    pub is_foreign_key: bool,
    pub description: String,
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i32>,
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<i32>,
    #[serde(rename = "databaseTableId")]
    pub database_table_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatabaseColumnDto {
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "dataType")]
    pub data_type: Option<String>,
    #[serde(rename = "isNullable")]
    pub is_nullable: Option<bool>,
    #[serde(rename = "isPrimaryKey")]
    pub is_primary_key: Option<bool>,
    #[serde(rename = "isForeignKey")]
    pub is_foreign_key: Option<bool>,
    pub description: Option<String>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<i32>,
    #[serde(rename = "minLength")]
    pub min_length: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseColumnEntity {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "dataType")]
    pub data_type: String,
    
    #[serde(rename = "isNullable")]
    pub is_nullable: bool,
    #[serde(rename = "isPrimaryKey")]
    pub is_primary_key: bool,
    #[serde(rename = "isForeignKey")]
    pub is_foreign_key: bool,
    pub description: String,
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i32>,
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<i32>,
    #[serde(rename = "databaseTableId")]
    pub database_table_id: DatabaseTableReference,
    #[serde(rename = "company_id")]
    pub company_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTableReference {
    pub id: String,
    pub name: String,
    pub description: String,
    pub table_reference: Option<String>,
    pub table_type: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    pub company_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatabaseTableDto {
    pub name: String,
    pub description: String,
    pub table_reference: Option<String>,
    pub table_type: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<CreateDatabaseColumnDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatabaseTableDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub table_reference: Option<String>,
    pub table_type: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: Option<String>,
    pub resource: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTableEntity {
    pub id: String,
    pub name: String,
    pub description: String,
    pub table_reference: Option<String>,
    pub table_type: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    pub company_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<DatabaseColumnEntity>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceItemDto {
    pub name: String,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatabaseViewDto {
    pub name: String,
    pub description: String,
    pub resource: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "mainResource")]
    pub main_resource: Option<String>,
    #[serde(rename = "isFhirDestination")]
    pub is_fhir_destination: Option<bool>,
    #[serde(rename = "isInterHealthDestination")]
    pub is_interhealth_destination: Option<bool>,
    #[serde(rename = "databaseConfigurationId")]
    pub database_configuration_id: String,
    #[serde(rename = "targetIntegrationId", skip_serializing_if = "Option::is_none")]
    pub target_integration_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<ResourceItemDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatabaseViewDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub resource: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: Option<String>,
    #[serde(rename = "mainResource")]
    pub main_resource: Option<String>,
    #[serde(rename = "isFhirDestination")]
    pub is_fhir_destination: Option<bool>,
    #[serde(rename = "isInterHealthDestination")]
    pub is_interhealth_destination: Option<bool>,
    #[serde(rename = "databaseConfigurationId")]
    pub database_configuration_id: Option<String>,
    #[serde(rename = "targetIntegrationId", skip_serializing_if = "Option::is_none")]
    pub target_integration_id: Option<String>,
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<ResourceItemDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseViewEntity {
    pub id: String,
    pub name: String,
    pub description: String,
    pub resource: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "mainResource")]
    pub main_resource: Option<String>,
    #[serde(rename = "isFhirDestination")]
    pub is_fhir_destination: bool,
    #[serde(rename = "isInterHealthDestination")]
    pub is_interhealth_destination: bool,
    #[serde(rename = "databaseConfigurationId")]
    pub database_configuration_id: String,
    #[serde(rename = "targetIntegrationId", skip_serializing_if = "Option::is_none")]
    pub target_integration_id: Option<String>,
    pub company_id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<ResourceItemDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfigurationReference {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub db_type: String,
    pub host: String,
    pub port: i32,
    pub database: String,
    pub username: String,
    pub password: String,
    pub company_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelValue {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDatabaseModelDto {
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(default)]
    pub values: Vec<ModelValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatabaseModelDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub values: Option<Vec<ModelValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseModelEntity {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<ModelValue>>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingValueItemEntity {
    pub source_key: String,
    pub source_description: String,
    pub status: String,
    pub company_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseModelValueEntity {
    pub id: String,
    pub owner_id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub code: String,
    pub description: String,
    pub clients: Vec<MappingValueItemEntity>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpsertDatabaseModelValueDto {
    #[serde(rename = "type")]
    pub type_field: String,
    pub source_key: String,
    pub source_description: String,
    pub code: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateDatabaseModelValueClientMappingDto {
    pub connection_id: Option<String>,
    pub source_key: Option<String>,
    pub source_description: Option<String>,
    pub status: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
}
