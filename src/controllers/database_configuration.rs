use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::application::{AppState, DatabaseConfigurationUseCase};
use crate::core::AuthUser;
use crate::domain::dtos::{CreateDatabaseConfigurationDto, UpdateDatabaseConfigurationDto, DatabaseConfigurationEntity};
use crate::utils::{ApiResponse, AppResult, PaginationResponse, PaginationQuery};

pub async fn create_database_configuration(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(mut payload): Json<CreateDatabaseConfigurationDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseConfigurationEntity>>)> {
    payload.company_id = Some(auth.company_id);
    let use_case = DatabaseConfigurationUseCase::new(state.database_configuration_repository.clone());
    let config = use_case.create_database_configuration(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Conector criado com sucesso", config)),
    ))
}

pub async fn get_all_database_configurations(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<PaginationResponse<DatabaseConfigurationEntity>>> {
    let use_case = DatabaseConfigurationUseCase::new(state.database_configuration_repository.clone());
    let result = use_case
        .get_all_database_configurations(pagination.currentPage, pagination.itemsPerPage)
        .await?;

    Ok(Json(result))
}

pub async fn get_database_configuration_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<DatabaseConfigurationEntity>>> {
    let use_case = DatabaseConfigurationUseCase::new(state.database_configuration_repository.clone());
    let config = use_case.get_database_configuration_by_id(&id).await?;

    Ok(Json(ApiResponse::success("Conector encontrado", config)))
}

pub async fn test_connection(
    State(state): State<AppState>,
    Json(payload): Json<CreateDatabaseConfigurationDto>,
) -> AppResult<Json<serde_json::Value>> {
    let use_case = DatabaseConfigurationUseCase::new(state.database_configuration_repository.clone());
    let result = use_case.test_connection(payload).await?;

    Ok(Json(result))
}

pub async fn update_database_configuration(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDatabaseConfigurationDto>,
) -> AppResult<Json<ApiResponse<DatabaseConfigurationEntity>>> {
    let use_case = DatabaseConfigurationUseCase::new(state.database_configuration_repository.clone());
    let config = use_case.update_database_configuration(&id, payload).await?;

    Ok(Json(ApiResponse::success(
        "Conector atualizado com sucesso",
        config,
    )))
}

pub async fn delete_database_configuration(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = DatabaseConfigurationUseCase::new(state.database_configuration_repository.clone());
    use_case.delete_database_configuration(&id).await?;

    Ok(Json(ApiResponse::success("Conector excluído com sucesso", "Deleted".to_string())))
}
