use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUserDto {
    pub items_per_page: Option<i32>,
    pub items_to_skip: Option<i32>,
    pub current_page: Option<i32>,
    pub filter: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUserResponse {
    pub user_code: String,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOneUserDto {
    pub user_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOneUserResponse {
    pub user_code: String,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEntity {
    pub id: String,
    pub name: String,
    pub email: String,
    pub status: bool,
    #[serde(rename = "type")]
    pub user_type: String,
    pub primary_document: Option<String>,
    pub company: Option<CompanyInfo>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    pub id: String,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
    pub password: String,
    #[serde(rename = "type")]
    pub user_type: Option<String>,
    pub primary_document: Option<String>,
    #[serde(default = "default_user_status")]
    pub status: bool,
    #[serde(rename = "companyId")]
    pub company_id: Option<String>,
}

fn default_user_status() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserDto {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub status: Option<bool>,
    #[serde(rename = "type")]
    pub user_type: Option<String>,
    pub primary_document: Option<String>,
    #[serde(rename = "companyId")]
    pub company_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStatusDto {
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTypeDto {
    #[serde(rename = "type")]
    pub user_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignCompanyDto {
    pub company_id: String,
}
