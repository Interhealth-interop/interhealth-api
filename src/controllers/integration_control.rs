use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::application::{AppState, IntegrationControlUseCase};
use crate::core::AuthUser;
use crate::domain::dtos::{
    CreateIntegrationControlDto, IntegrationControlEntity, UpdateIntegrationControlDto,
};
use crate::utils::{ApiResponse, AppResult};

pub async fn create_integration_control(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateIntegrationControlDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<IntegrationControlEntity>>)> {
    let use_case = IntegrationControlUseCase::new(state.integration_control_repository.clone());

    let created = use_case
        .create_integration_control(payload, auth.company_id)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "Integration control created successfully",
            created,
        )),
    ))
}

pub async fn get_integration_control_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<IntegrationControlEntity>>> {
    let use_case = IntegrationControlUseCase::new(state.integration_control_repository.clone());

    let control = use_case.get_integration_control_by_id(&id).await?;
    Ok(Json(ApiResponse::success("Integration control found", control)))
}

pub async fn get_integration_controls_by_database_view_id(
    State(state): State<AppState>,
    Path(view_id): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<IntegrationControlEntity>>>> {
    let use_case = IntegrationControlUseCase::new(state.integration_control_repository.clone());

    let controls = use_case
        .get_integration_controls_by_database_view_id(&view_id)
        .await?;

    Ok(Json(ApiResponse::success(
        "Integration controls retrieved successfully",
        controls,
    )))
}

pub async fn update_integration_control(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateIntegrationControlDto>,
) -> AppResult<Json<ApiResponse<IntegrationControlEntity>>> {
    let use_case = IntegrationControlUseCase::new(state.integration_control_repository.clone());

    let updated = use_case.update_integration_control(&id, payload).await?;

    Ok(Json(ApiResponse::success(
        "Integration control updated successfully",
        updated,
    )))
}

pub async fn delete_integration_control(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = IntegrationControlUseCase::new(state.integration_control_repository.clone());

    use_case.delete_integration_control(&id).await?;

    Ok(Json(ApiResponse::success(
        "Integration control deleted successfully",
        "Deleted".to_string(),
    )))
}
