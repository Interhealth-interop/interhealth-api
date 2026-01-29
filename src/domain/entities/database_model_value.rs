use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatabaseModelValueClient {
    pub source_key: String,
    pub source_description: String,
    pub status: String,
    pub company_id: ObjectId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatabaseModelValue {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<ObjectId>,
    pub owner_id: ObjectId,
    #[serde(rename = "type")]
    pub type_field: String,
    pub code: String,
    pub description: String,
    pub clients: Vec<DatabaseModelValueClient>,
    #[serde(with = "crate::utils::utils::date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::utils::utils::date_format")]
    pub updated_at: DateTime<Utc>,
}
