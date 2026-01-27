use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json
};
use serde::Deserialize;

use crate::{
    application::{AppState, DatabaseTransformationUseCase},
    core::AuthUser,
    domain::dtos::{DatabaseTransformationEntity, CreateDatabaseTransformationDto, UpdateDatabaseTransformationDto, AvailableModelForTransformationDto},
    utils::{ApiResponse, AppResult, PaginationResponse, PaginationQuery},
};

#[derive(Debug, Deserialize)]
pub struct DatabaseTransformationQueryParams {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(rename = "type")]
    pub type_filter: Option<String>,
    #[serde(default)]
    pub values: Option<bool>,
}

pub async fn create_database_transformation(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(mut payload): Json<CreateDatabaseTransformationDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseTransformationEntity>>)> {
    payload.company_id = auth.company_id;
    let use_case = DatabaseTransformationUseCase::new(state.database_transformation_repository.clone());

    let transformation = use_case.create_database_transformation(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Modelo de dado criado com sucesso", transformation)),
    ))
}

pub async fn get_all_database_transformations(
    State(state): State<AppState>,
    Query(params): Query<DatabaseTransformationQueryParams>,
) -> AppResult<Json<PaginationResponse<DatabaseTransformationEntity>>> {
    let use_case = DatabaseTransformationUseCase::new(state.database_transformation_repository.clone());
    let include_values = params.values.unwrap_or(true);
    let result = use_case
        .get_all_database_transformations(
            params.pagination.currentPage,
            params.pagination.itemsPerPage,
            params.type_filter,
            include_values,
        )
        .await?;

    Ok(Json(result))
}

pub async fn get_database_transformation_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<DatabaseTransformationEntity>>> {
    let use_case = DatabaseTransformationUseCase::new(state.database_transformation_repository.clone());
    let transformation = use_case.get_database_transformation_by_id(&id).await?;

    Ok(Json(ApiResponse::success("Modelo de dado encontrado", transformation)))
}

pub async fn update_database_transformation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDatabaseTransformationDto>,
) -> AppResult<Json<ApiResponse<DatabaseTransformationEntity>>> {
    let use_case = DatabaseTransformationUseCase::new(state.database_transformation_repository.clone());
    let transformation = use_case.update_database_transformation(&id, payload).await?;

    Ok(Json(ApiResponse::success(
        "Modelo de dado atualizado com sucesso",
        transformation,
    )))
}

pub async fn delete_database_transformation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = DatabaseTransformationUseCase::new(state.database_transformation_repository.clone());
    use_case.delete_database_transformation(&id).await?;

    Ok(Json(ApiResponse::success("Modelo de dado excluído com sucesso", "Excluído".to_string())))
}

pub async fn get_available_models_for_transformation(
    State(state): State<AppState>,
    Query(params): Query<DatabaseTransformationQueryParams>,
) -> AppResult<Json<PaginationResponse<AvailableModelForTransformationDto>>> {
    let use_case = DatabaseTransformationUseCase::new(state.database_transformation_repository.clone());
    let result = use_case
        .get_available_models_for_transformation(
            state.database_model_repository.clone(),
            params.pagination.currentPage,
            params.pagination.itemsPerPage,
            params.type_filter,
        )
        .await?;

    Ok(Json(result))
}
