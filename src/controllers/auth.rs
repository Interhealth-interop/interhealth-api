use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::application::{AppState, AuthUseCase};
use crate::domain::dtos::{AuthEntity, LoginDto, RegisterDto};
use crate::utils::{ApiResponse, AppResult};

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> AppResult<Json<ApiResponse<AuthEntity>>> {
    let use_case = AuthUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
        state.jwt_service.clone(),
    );
    
    let auth_data = use_case.login(payload).await?;

    Ok(Json(ApiResponse::success("Login realizado com sucesso", auth_data)))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterDto>,
) -> AppResult<Json<ApiResponse<AuthEntity>>> {
    let use_case = AuthUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
        state.jwt_service.clone(),
    );
    
    let auth_data = use_case.register(payload).await?;

    Ok(Json(ApiResponse::success("Registro realizado com sucesso", auth_data)))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<Json<ApiResponse<AuthEntity>>> {
    let use_case = AuthUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
        state.jwt_service.clone(),
    );
    
    let auth_data = use_case.refresh_token(&payload.refresh_token).await?;

    Ok(Json(ApiResponse::success("Token atualizado com sucesso", auth_data)))
}
