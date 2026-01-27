use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json
};
use serde::Deserialize;

use crate::{
    application::{AppState, DatabaseModelUseCase},
    domain::dtos::{DatabaseModelEntity, CreateDatabaseModelDto, UpdateDatabaseModelDto},
    utils::{ApiResponse, AppResult, PaginationResponse, PaginationQuery},
};

#[derive(Debug, Deserialize)]
pub struct DatabaseModelQueryParams {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(rename = "type")]
    pub type_filter: Option<String>,
    #[serde(default)]
    pub values: Option<bool>,
}

pub async fn create_database_model(
    State(state): State<AppState>,
    Json(payload): Json<CreateDatabaseModelDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseModelEntity>>)> {
    let use_case = DatabaseModelUseCase::new(state.database_model_repository.clone());

    let model = use_case.create_database_model(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Modelo de terminologia criado com sucesso", model)),
    ))
}

pub async fn get_all_database_models(
    State(state): State<AppState>,
    Query(params): Query<DatabaseModelQueryParams>,
) -> AppResult<Json<PaginationResponse<DatabaseModelEntity>>> {
    let use_case = DatabaseModelUseCase::new(state.database_model_repository.clone());
    let include_values = params.values.unwrap_or(true);
    let result = use_case
        .get_all_database_models(
            params.pagination.currentPage,
            params.pagination.itemsPerPage,
            params.type_filter,
            include_values,
        )
        .await?;

    Ok(Json(result))
}

pub async fn get_database_model_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<DatabaseModelEntity>>> {
    let use_case = DatabaseModelUseCase::new(state.database_model_repository.clone());
    let model = use_case.get_database_model_by_id(&id).await?;

    Ok(Json(ApiResponse::success("Modelo de terminologia encontrado", model)))
}

pub async fn update_database_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDatabaseModelDto>,
) -> AppResult<Json<ApiResponse<DatabaseModelEntity>>> {
    let use_case = DatabaseModelUseCase::new(state.database_model_repository.clone());
    let model = use_case.update_database_model(&id, payload).await?;

    Ok(Json(ApiResponse::success(
        "Modelo de terminologia atualizado com sucesso",
        model,
    )))
}

pub async fn delete_database_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = DatabaseModelUseCase::new(state.database_model_repository.clone());
    use_case.delete_database_model(&id).await?;

    Ok(Json(ApiResponse::success("Modelo de terminologia excluído com sucesso", "Excluído".to_string())))
}
