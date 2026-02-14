use std::sync::Arc;

use crate::domain::dtos::{UserEntity, CompanyInfo, CreateUserDto, UpdateUserDto};
use crate::infrastructure::repositories::{UserRepository, CompanyRepository};
use crate::utils::{AppError, AppResult};

pub struct UserUseCase {
    user_repository: Arc<UserRepository>,
    company_repository: Arc<CompanyRepository>,
}

impl UserUseCase {
    pub fn new(
        user_repository: Arc<UserRepository>,
        company_repository: Arc<CompanyRepository>,
    ) -> Self {
        Self {
            user_repository,
            company_repository,
        }
    }

    async fn map_user_to_entity(&self, user: crate::domain::entities::User) -> AppResult<UserEntity> {
        let user_id = user.id.ok_or_else(|| AppError::InternalServerError)?.to_hex();
        
        let company = if let Some(ref company_id_str) = user.company_id {
            match self.company_repository.find_by_id(company_id_str).await {
                Ok(Some(c)) => {
                    Some(CompanyInfo {
                        id: c.id.ok_or_else(|| AppError::InternalServerError)?.to_hex(),
                        code: c.code,
                        name: c.name,
                    })
                }
                Ok(None) => {
                    None
                }
                Err(e) => {
                    None
                }
            }
        } else {
            None
        };

        Ok(UserEntity {
            id: user_id,
            name: user.name,
            email: user.email,
            status: user.status,
            user_type: user.user_type,
            primary_document: user.primary_document,
            company,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        })
    }

    pub async fn create_user(&self, data: CreateUserDto) -> AppResult<UserEntity> {
        if let Some(ref company_id_str) = data.company_id {
            let company = self.company_repository.find_by_id(company_id_str).await?;
            if company.is_none() {
                return Err(AppError::NotFound("Invalid company ID".to_string()));
            }
        }

        let user_type = data.user_type.unwrap_or_else(|| "USER".to_string());

        let user = self.user_repository.create(crate::infrastructure::repositories::user::CreateUserDto {
            name: data.name,
            email: data.email,
            password: data.password,
            user_type,
            primary_document: data.primary_document,
            status: data.status,
            company_id: data.company_id,
        }).await?;

        self.map_user_to_entity(user).await
    }

    pub async fn get_all_users(&self, page: i64, limit: i64, order_field: Option<String>, order_by: Option<String>) -> AppResult<(Vec<UserEntity>, i64)> {
        use crate::utils::sort_helper::build_sort_document;
        
        let sort_document = build_sort_document(order_field, order_by);
        let (users, total) = self.user_repository.find_all(page, limit, sort_document).await?;
        
        let mut user_entities = Vec::new();
        for user in users {
            user_entities.push(self.map_user_to_entity(user).await?);
        }

        Ok((user_entities, total))
    }

    pub async fn get_user_by_id(&self, id: &str) -> AppResult<UserEntity> {
        let user = self.user_repository.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        self.map_user_to_entity(user).await
    }

    pub async fn get_user_by_email(&self, email: &str) -> AppResult<UserEntity> {
        let user = self.user_repository.find_by_email(email).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        self.map_user_to_entity(user).await
    }

    pub async fn get_users_by_company(&self, company_id: &str, page: i64, limit: i64) -> AppResult<(Vec<UserEntity>, i64)> {
        let (users, total) = self.user_repository.find_by_company(company_id, page, limit).await?;
        
        let mut user_entities = Vec::new();
        for user in users {
            user_entities.push(self.map_user_to_entity(user).await?);
        }

        Ok((user_entities, total))
    }

    pub async fn update_user(&self, id: &str, data: UpdateUserDto) -> AppResult<UserEntity> {
        if let Some(ref company_id_str) = data.company_id {
            let company = self.company_repository.find_by_id(company_id_str).await?;
            if company.is_none() {
                return Err(AppError::NotFound("Invalid company ID".to_string()));
            }
        }

        let user = self.user_repository.update(id, crate::infrastructure::repositories::user::UpdateUserDto {
            name: data.name,
            email: data.email,
            password: data.password,
            status: data.status,
            user_type: data.user_type,
            primary_document: data.primary_document,
            company_id: data.company_id,
        }).await?;

        self.map_user_to_entity(user).await
    }

    pub async fn change_user_status(&self, id: &str, status: bool) -> AppResult<UserEntity> {
        let user = self.user_repository.change_status(id, status).await?;
        self.map_user_to_entity(user).await
    }

    pub async fn change_user_type(&self, id: &str, user_type: &str) -> AppResult<UserEntity> {
        let user = self.user_repository.change_type(id, user_type).await?;
        self.map_user_to_entity(user).await
    }

    pub async fn assign_user_to_company(&self, id: &str, company_id: &str) -> AppResult<UserEntity> {
        let company = self.company_repository.find_by_id(company_id).await?;
        if company.is_none() {
            return Err(AppError::NotFound("Invalid company ID".to_string()));
        }

        let user = self.user_repository.assign_company(id, company_id).await?;
        self.map_user_to_entity(user).await
    }

    pub async fn remove_user_from_company(&self, id: &str) -> AppResult<UserEntity> {
        let user = self.user_repository.remove_company(id).await?;
        self.map_user_to_entity(user).await
    }

    pub async fn delete_user(&self, id: &str) -> AppResult<bool> {
        self.user_repository.delete(id).await
    }
}
