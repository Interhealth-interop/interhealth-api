use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use crate::utils::utils::{date_format, object_id_format};

fn default_user_type() -> String {
    "USER".to_string()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        with = "object_id_format"
    )]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub status: bool,
    #[serde(default = "default_user_type", rename = "type")]
    pub user_type: String,
    pub primary_document: Option<String>,
    pub company_id: Option<String>,
    #[serde(with = "date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "date_format")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserType {
    Master,
    Admin,
    User,
}

impl UserType {
    pub fn as_str(&self) -> &str {
        match self {
            UserType::Master => "MASTER",
            UserType::Admin => "ADMIN",
            UserType::User => "USER",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "MASTER" => Some(UserType::Master),
            "ADMIN" => Some(UserType::Admin),
            "USER" => Some(UserType::User),
            _ => None,
        }
    }
}
