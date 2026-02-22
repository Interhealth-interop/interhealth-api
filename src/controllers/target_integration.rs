use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::application::{AppState, TargetIntegrationUseCase};
use crate::core::AuthUser;
use crate::domain::dtos::{
    CreateTargetIntegrationDto, TargetIntegrationEntity, UpdateTargetIntegrationDto,
};
use crate::utils::{ApiResponse, AppResult};

pub async fn create_target_integration(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateTargetIntegrationDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<TargetIntegrationEntity>>)> {
    let use_case = TargetIntegrationUseCase::new(
        state.target_integration_repository.clone(),
        state.database_view_repository.clone(),
    );

    let created = use_case
        .create_target_integration(payload, auth.company_id)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "Target integration created successfully",
            created,
        )),
    ))
}

pub async fn get_target_integration_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<TargetIntegrationEntity>>> {
    let use_case = TargetIntegrationUseCase::new(
        state.target_integration_repository.clone(),
        state.database_view_repository.clone(),
    );

    let target = use_case.get_target_integration_by_id(&id).await?;
    Ok(Json(ApiResponse::success("Target integration found", target)))
}

pub async fn get_target_integration_by_database_view_id(
    State(state): State<AppState>,
    Path(view_id): Path<String>,
) -> AppResult<Json<ApiResponse<TargetIntegrationEntity>>> {
    let use_case = TargetIntegrationUseCase::new(
        state.target_integration_repository.clone(),
        state.database_view_repository.clone(),
    );

    let target = use_case
        .get_target_integration_by_database_view_id(&view_id)
        .await?;

    Ok(Json(ApiResponse::success("Target integration found", target)))
}

pub async fn update_target_integration(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTargetIntegrationDto>,
) -> AppResult<Json<ApiResponse<TargetIntegrationEntity>>> {
    let use_case = TargetIntegrationUseCase::new(
        state.target_integration_repository.clone(),
        state.database_view_repository.clone(),
    );

    let target = use_case.update_target_integration(&id, payload).await?;

    Ok(Json(ApiResponse::success(
        "Target integration updated successfully",
        target,
    )))
}

pub async fn delete_target_integration(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = TargetIntegrationUseCase::new(
        state.target_integration_repository.clone(),
        state.database_view_repository.clone(),
    );

    use_case.delete_target_integration(&id).await?;

    Ok(Json(ApiResponse::success(
        "Target integration deleted successfully",
        "Deleted".to_string(),
    )))
}
