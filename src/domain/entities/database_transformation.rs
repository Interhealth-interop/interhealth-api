use std::collections::HashMap;

use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValueMappingItem {
    pub code: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatabaseTransformation {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<ObjectId>,
    pub value_mappings: HashMap<String, ValueMappingItem>,
    #[serde(with = "crate::utils::utils::date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::utils::utils::date_format")]
    pub updated_at: DateTime<Utc>,
}
