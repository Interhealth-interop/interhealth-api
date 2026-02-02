use serde::{Deserialize, Serialize};
use validator::Validate;
use std::collections::HashMap;

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
    pub port: i32,
    pub database: String,
    pub username: String,
    pub password: String,
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
    pub port: i32,
    pub database: String,
    pub username: String,
    pub password: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatabaseViewDto {
    pub name: String,
    pub description: String,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "isFhirDestination")]
    pub is_fhir_destination: Option<bool>,
    #[serde(rename = "isInterHealthDestination")]
    pub is_interhealth_destination: Option<bool>,
    #[serde(rename = "databaseConfigurationId")]
    pub database_configuration_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<ResourceItemDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatabaseViewDto {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: Option<String>,
    #[serde(rename = "isFhirDestination")]
    pub is_fhir_destination: Option<bool>,
    #[serde(rename = "isInterHealthDestination")]
    pub is_interhealth_destination: Option<bool>,
    #[serde(rename = "databaseConfigurationId")]
    pub database_configuration_id: Option<String>,
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<ResourceItemDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseViewEntity {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "isFhirDestination")]
    pub is_fhir_destination: bool,
    #[serde(rename = "isInterHealthDestination")]
    pub is_interhealth_destination: bool,
    #[serde(rename = "databaseConfigurationId")]
    pub database_configuration_id: String,
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
    #[serde(default)]
    pub values: Vec<ModelValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatabaseModelDto {
    pub name: Option<String>,
    pub description: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateDatabaseModelValueClientMappingDto {
    pub source_key: Option<String>,
    pub source_description: Option<String>,
    pub status: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
}
