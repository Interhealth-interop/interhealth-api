use serde::{Deserialize, Serialize};

use super::user::UserEntity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDto {
    pub name: String,
    pub email: String,
    pub password: String,
    #[serde(rename = "type")]
    pub user_type: Option<String>,
    pub company_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEntity {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
    #[serde(rename = "expiresIn")]
    pub expires_in: Option<i64>,
    pub user: Option<UserEntity>,
}
