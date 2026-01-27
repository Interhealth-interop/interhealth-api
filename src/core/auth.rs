use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode, HeaderMap},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::application::AppState;
use crate::utils::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub company_id: Option<String>,
    pub exp: u64,
    pub iat: u64,
}

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, user_id: &str, email: &str, user_type: &str, company_id: Option<String>, expires_in_seconds: u64) -> Result<String, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            user_type: user_type.to_string(),
            company_id,
            exp: now + expires_in_seconds,
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|_| AppError::InternalServerError)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))
    }
}

pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub user_type: String,
    pub company_id: String,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing Authorization header".to_string(),
                )
            })?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid Authorization header format".to_string(),
                )
            })?;

        let claims = state
            .jwt_service
            .verify_token(token)
            .map_err(|e| match e {
                AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Token verification failed".to_string()),
            })?;

        let company_id = claims.company_id.ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Token does not contain company_id".to_string(),
            )
        })?;

        Ok(AuthUser {
            user_id: claims.sub,
            email: claims.email,
            user_type: claims.user_type,
            company_id,
        })
    }
}
