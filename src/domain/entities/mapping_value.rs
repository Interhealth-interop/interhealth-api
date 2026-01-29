use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MappingValue {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<ObjectId>,
    pub owner_kind: String,
    pub owner_id: ObjectId,
    pub company_id: ObjectId,
    #[serde(rename = "type")]
    pub type_field: String,
    pub source_key: String,
    pub source_description: String,
    pub status: String,
    pub code: String,
    pub description: String,
    #[serde(with = "crate::utils::utils::date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::utils::utils::date_format")]
    pub updated_at: DateTime<Utc>,
}
