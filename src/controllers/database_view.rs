use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json
};
use serde::Deserialize;

use crate::application::{AppState, DatabaseViewUseCase};
use crate::core::AuthUser;
use crate::domain::dtos::{DatabaseViewEntity, CreateDatabaseViewDto, UpdateDatabaseViewDto};
use crate::utils::{ApiResponse, AppResult, PaginationResponse, PaginationQuery};

#[derive(Debug, Deserialize)]
pub struct DatabaseViewQueryParams {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(rename = "databaseConfigurationId")]
    pub database_configuration_id: Option<String>,
}

pub async fn create_database_view(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateDatabaseViewDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseViewEntity>>)> {
    let use_case = DatabaseViewUseCase::new(
        state.database_view_repository.clone(),
        state.database_configuration_repository.clone(),
        state.database_view_mapping_repository.clone(),
    );
    let view = use_case.create_database_view(payload, auth.company_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Integração criada com sucesso", view)),
    ))
}

pub async fn get_all_database_views(
    State(state): State<AppState>,
    Query(params): Query<DatabaseViewQueryParams>,
) -> AppResult<Json<PaginationResponse<DatabaseViewEntity>>> {
    let use_case = DatabaseViewUseCase::new(
        state.database_view_repository.clone(),
        state.database_configuration_repository.clone(),
        state.database_view_mapping_repository.clone(),
    );
    let result = use_case
        .get_all_database_views(
            params.pagination.currentPage,
            params.pagination.itemsPerPage,
            params.database_configuration_id,
            params.pagination.order_field,
            params.pagination.order_by,
        )
        .await?;

    Ok(Json(result))
}

pub async fn get_database_view_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<DatabaseViewEntity>>> {
    let use_case = DatabaseViewUseCase::new(
        state.database_view_repository.clone(),
        state.database_configuration_repository.clone(),
        state.database_view_mapping_repository.clone(),
    );
    let view = use_case.get_database_view_by_id(&id).await?;

    Ok(Json(ApiResponse::success("Integração encontrada", view)))
}


pub async fn update_database_view(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDatabaseViewDto>,
) -> AppResult<Json<ApiResponse<DatabaseViewEntity>>> {
    let use_case = DatabaseViewUseCase::new(
        state.database_view_repository.clone(),
        state.database_configuration_repository.clone(),
        state.database_view_mapping_repository.clone(),
    );
    let view = use_case.update_database_view(&id, payload).await?;

    Ok(Json(ApiResponse::success(
        "Integração atualizada com sucesso",
        view,
    )))
}

pub async fn delete_database_view(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = DatabaseViewUseCase::new(
        state.database_view_repository.clone(),
        state.database_configuration_repository.clone(),
        state.database_view_mapping_repository.clone(),
    );
    use_case.delete_database_view(&id).await?;

    Ok(Json(ApiResponse::success("Integração excluída com sucesso", "Excluído".to_string())))
}

pub async fn start_integration(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseViewEntity>>)> {
    let use_case = DatabaseViewUseCase::new(
        state.database_view_repository.clone(),
        state.database_configuration_repository.clone(),
        state.database_view_mapping_repository.clone(),
    );
    let view = use_case.start_integration(&id).await?;
    Ok((StatusCode::OK, Json(ApiResponse::success("Integração iniciada com sucesso", view))))
}

pub async fn cancel_integration(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseViewEntity>>)> {
    let use_case = DatabaseViewUseCase::new(
        state.database_view_repository.clone(),
        state.database_configuration_repository.clone(),
        state.database_view_mapping_repository.clone(),
    );
    let view = use_case.cancel_integration(&id).await?;
    Ok((StatusCode::OK, Json(ApiResponse::success("Integração cancelada com sucesso", view))))
}
