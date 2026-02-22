use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use crate::utils::utils::object_id_format;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetIntegration {
    #[serde(
        rename(serialize = "id", deserialize = "_id"),
        skip_serializing_if = "Option::is_none",
        with = "object_id_format"
    )]
    pub id: Option<ObjectId>,
    pub name: String,
    pub version: Option<String>,
    pub host: String,
    pub auth_type: Option<String>,
    pub credentials: Option<String>,
    #[serde(rename = "databaseViewId")]
    pub database_view_id: String,
    pub company_id: String,
    #[serde(with = "crate::utils::utils::date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::utils::utils::date_format")]
    pub updated_at: DateTime<Utc>,
}
