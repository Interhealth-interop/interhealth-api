use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use crate::utils::utils::object_id_format;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldMapping {
    #[serde(default, rename = "fieldOrigin", alias = "field_origin")]
    pub field_origin: String,
    #[serde(default, rename = "fieldDestiny", alias = "field_destiny")]
    pub field_destiny: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "referenceDestiny", skip_serializing_if = "Option::is_none")]
    pub reference_destiny: Option<std::collections::HashMap<String, String>>,
    #[serde(default, rename = "relationshipDestiny", skip_serializing_if = "Option::is_none")]
    pub relationship_destiny: Option<String>,
    #[serde(default, rename = "dataType", alias = "data_type")]
    pub data_type: String,
    #[serde(default, rename = "isNullable", alias = "is_nullable")]
    pub is_nullable: bool,
    #[serde(default, rename = "minLength", alias = "min_length")]
    pub min_length: i32,
    #[serde(default, rename = "maxLength", alias = "max_length")]
    pub max_length: i32,
    #[serde(default, rename = "isEnumerable", alias = "is_enumerable")]
    pub is_enumerable: bool,
    #[serde(default, rename = "transformationId", alias = "transformation_id", skip_serializing_if = "Option::is_none")]
    pub transformation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatabaseViewMapping {
    #[serde(
        rename(serialize = "id", deserialize = "_id"),
        skip_serializing_if = "Option::is_none",
        with = "object_id_format"
    )]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "entityType", alias = "entity_type")]
    pub entity_type: String,
    #[serde(default, rename = "databaseTableOriginId", alias = "database_table_origin_id")]
    pub database_table_origin_id: String,
    #[serde(default, rename = "databaseTableDestinyId", alias = "database_table_destiny_id")]
    pub database_table_destiny_id: String,
    #[serde(default, rename = "dataViewId", alias = "data_view_id")]
    pub data_view_id: String,
    #[serde(default, rename = "fieldMappings", alias = "field_mappings")]
    pub field_mappings: Vec<FieldMapping>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(with = "crate::utils::utils::date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::utils::utils::date_format")]
    pub updated_at: DateTime<Utc>,
}

fn default_status() -> String {
    "draft".to_string()
}
