use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use crate::utils::utils::{date_format, object_id_format};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceItem {
    pub name: String,
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatabaseView {
    #[serde(
        rename(serialize = "id", deserialize = "_id"),
        skip_serializing_if = "Option::is_none",
        with = "object_id_format"
    )]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: String,
    pub resource: Option<String>,
    pub entity_type: String,
    pub main_resource: Option<String>,
    pub is_fhir_destination: Option<bool>,
    pub is_interhealth_destination: Option<bool>,
    pub database_configuration_id: String,
    pub company_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_integration_id: Option<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<ResourceItem>>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "crate::utils::utils::optional_date_format")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "crate::utils::utils::optional_date_format")]
    pub cancelled_at: Option<DateTime<Utc>>,
    #[serde(with = "crate::utils::utils::date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::utils::utils::date_format")]
    pub updated_at: DateTime<Utc>,
}
