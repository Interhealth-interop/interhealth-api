use std::sync::Arc;

use crate::domain::dtos::{AuthEntity, LoginDto, RegisterDto, UserEntity, CompanyInfo};
use crate::infrastructure::repositories::{UserRepository, CompanyRepository};
use crate::utils::{AppError, AppResult};
use crate::core::JwtService;

pub struct AuthUseCase {
    user_repository: Arc<UserRepository>,
    company_repository: Arc<CompanyRepository>,
    jwt_service: Arc<JwtService>,
}

impl AuthUseCase {
    pub fn new(
        user_repository: Arc<UserRepository>,
        company_repository: Arc<CompanyRepository>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            user_repository,
            company_repository,
            jwt_service,
        }
    }

    pub async fn login(&self, login_dto: LoginDto) -> AppResult<AuthEntity> {
        let user = self.user_repository.find_by_email(&login_dto.email).await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        if !self.user_repository.validate_password(&login_dto.password, &user.password) {
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        if !user.status {
            return Err(AppError::Unauthorized("User account is disabled".to_string()));
        }

        let user_id = user.id.ok_or_else(|| AppError::InternalServerError)?.to_hex();
        
        let access_token = self.jwt_service.generate_token(
            &user_id,
            &user.email,
            &user.user_type,
            user.company_id.clone(),
            3600,
        )?;

        let refresh_token = self.jwt_service.generate_token(
            &user_id,
            &user.email,
            &user.user_type,
            user.company_id.clone(),
            604800,
        )?;

        let company = if let Some(ref company_id_str) = user.company_id {
            if let Some(c) = self.company_repository.find_by_id(company_id_str).await? {
                Some(CompanyInfo {
                    id: c.id.ok_or_else(|| AppError::InternalServerError)?.to_hex(),
                    code: c.code,
                    name: c.name,
                })
            } else {
                None
            }
        } else {
            None
        };

        let user_entity = UserEntity {
            id: user_id,
            name: user.name,
            email: user.email,
            status: user.status,
            user_type: user.user_type,
            primary_document: user.primary_document,
            company,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        };

        Ok(AuthEntity {
            access_token,
            refresh_token: Some(refresh_token),
            expires_in: Some(3600),
            user: Some(user_entity),
        })
    }

    pub async fn register(&self, register_dto: RegisterDto) -> AppResult<AuthEntity> {
        let existing_user = self.user_repository.find_by_email(&register_dto.email).await?;
        if existing_user.is_some() {
            return Err(AppError::Conflict("User with this email already exists".to_string()));
        }

        if let Some(ref company_id_str) = register_dto.company_id {
            let company = self.company_repository.find_by_id(company_id_str).await?;
            if company.is_none() {
                return Err(AppError::NotFound("Invalid company ID".to_string()));
            }
        }

        let user_type = register_dto.user_type.unwrap_or_else(|| "USER".to_string());

        let new_user = self.user_repository.create(crate::infrastructure::repositories::user::CreateUserDto {
            name: register_dto.name,
            email: register_dto.email,
            password: register_dto.password,
            user_type: user_type.clone(),
            primary_document: None,
            status: true,
            company_id: register_dto.company_id.clone(),
        }).await?;

        let new_user_id = new_user.id.ok_or_else(|| AppError::InternalServerError)?.to_hex();

        let access_token = self.jwt_service.generate_token(
            &new_user_id,
            &new_user.email,
            &new_user.user_type,
            new_user.company_id.clone(),
            3600,
        )?;

        let refresh_token = self.jwt_service.generate_token(
            &new_user_id,
            &new_user.email,
            &new_user.user_type,
            new_user.company_id.clone(),
            604800,
        )?;

        let company = if let Some(ref company_id_str) = new_user.company_id {
            if let Some(c) = self.company_repository.find_by_id(company_id_str).await? {
                Some(CompanyInfo {
                    id: c.id.ok_or_else(|| AppError::InternalServerError)?.to_hex(),
                    code: c.code,
                    name: c.name,
                })
            } else {
                None
            }
        } else {
            None
        };

        let user_entity = UserEntity {
            id: new_user_id,
            name: new_user.name,
            email: new_user.email,
            status: new_user.status,
            user_type: new_user.user_type,
            primary_document: new_user.primary_document,
            company,
            created_at: new_user.created_at.to_rfc3339(),
            updated_at: new_user.updated_at.to_rfc3339(),
        };

        Ok(AuthEntity {
            access_token,
            refresh_token: Some(refresh_token),
            expires_in: Some(3600),
            user: Some(user_entity),
        })
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> AppResult<AuthEntity> {
        let claims = self.jwt_service.verify_token(refresh_token)?;

        let user = self.user_repository.find_by_id(&claims.sub).await?
            .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".to_string()))?;

        if !user.status {
            return Err(AppError::Unauthorized("User account is disabled".to_string()));
        }

        let user_id = user.id.ok_or_else(|| AppError::InternalServerError)?.to_hex();

        let access_token = self.jwt_service.generate_token(
            &user_id,
            &user.email,
            &user.user_type,
            user.company_id.clone(),
            3600,
        )?;

        let new_refresh_token = self.jwt_service.generate_token(
            &user_id,
            &user.email,
            &user.user_type,
            user.company_id.clone(),
            604800,
        )?;

        let company = if let Some(ref company_id_str) = user.company_id {
            if let Some(c) = self.company_repository.find_by_id(company_id_str).await? {
                Some(CompanyInfo {
                    id: c.id.ok_or_else(|| AppError::InternalServerError)?.to_hex(),
                    code: c.code,
                    name: c.name,
                })
            } else {
                None
            }
        } else {
            None
        };

        let user_entity = UserEntity {
            id: user_id,
            name: user.name,
            email: user.email,
            status: user.status,
            user_type: user.user_type,
            primary_document: user.primary_document,
            company,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        };

        Ok(AuthEntity {
            access_token,
            refresh_token: Some(new_refresh_token),
            expires_in: Some(3600),
            user: Some(user_entity),
        })
    }

    pub async fn validate_user(&self, user_id: &str) -> AppResult<Option<UserEntity>> {
        let user = self.user_repository.find_by_id(user_id).await?;

        if let Some(user) = user {
            if !user.status {
                return Ok(None);
            }

            let user_id_hex = user.id.ok_or_else(|| AppError::InternalServerError)?.to_hex();

            let company = if let Some(ref company_id_str) = user.company_id {
                if let Some(c) = self.company_repository.find_by_id(company_id_str).await? {
                    Some(CompanyInfo {
                        id: c.id.ok_or_else(|| AppError::InternalServerError)?.to_hex(),
                        code: c.code,
                        name: c.name,
                    })
                } else {
                    None
                }
            } else {
                None
            };

            return Ok(Some(UserEntity {
                id: user_id_hex,
                name: user.name,
                email: user.email,
                status: user.status,
                user_type: user.user_type,
                primary_document: user.primary_document,
                company,
                created_at: user.created_at.to_rfc3339(),
                updated_at: user.updated_at.to_rfc3339(),
            }));
        }

        Ok(None)
    }
}
