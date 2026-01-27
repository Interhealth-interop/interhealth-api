use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyEntity {
    pub id: String,
    pub code: String,
    pub name: String,
    pub cnpj: String,
    pub address: Option<String>,
    pub number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zipcode: Option<String>,
    pub country: Option<String>,
    pub status: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompanyDto {
    pub code: String,
    pub name: String,
    pub cnpj: String,
    pub address: Option<String>,
    pub number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zipcode: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCompanyDto {
    pub code: Option<String>,
    pub name: Option<String>,
    pub cnpj: Option<String>,
    pub address: Option<String>,
    pub number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zipcode: Option<String>,
    pub country: Option<String>,
    pub status: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeCompanyStatusDto {
    pub status: bool,
}
