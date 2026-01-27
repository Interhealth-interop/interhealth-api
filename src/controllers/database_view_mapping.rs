use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json
};
use serde::Deserialize;
use serde_json::Value;

use crate::application::{
    AppState, DatabaseViewMappingUseCase, DatabaseViewMappingEntity,
    CreateDatabaseViewMappingDto, UpdateDatabaseViewMappingDto,
};
use crate::core::AuthUser;
use crate::utils::{ApiResponse, AppResult, PaginationResponse, PaginationQuery};

#[derive(Debug, Deserialize)]
pub struct DatabaseViewMappingQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(rename = "databaseTableOriginId")]
    pub database_table_origin_id: Option<String>,
    #[serde(rename = "databaseTableDestinyId")]
    pub database_table_destiny_id: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: Option<String>,
    #[serde(rename = "dataViewId")]
    pub data_view_id: Option<String>,
}

pub async fn create_database_view_mapping(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateDatabaseViewMappingDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseViewMappingEntity>>)> {
    let use_case = DatabaseViewMappingUseCase::new(state.database_view_mapping_repository.clone());
    let mapping = use_case.create_database_view_mapping(payload, auth.company_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Mapeamento criado com sucesso", mapping)),
    ))
}

pub async fn get_all_database_view_mappings(
    State(state): State<AppState>,
    Query(query): Query<DatabaseViewMappingQuery>,
) -> AppResult<Json<PaginationResponse<DatabaseViewMappingEntity>>> {
    let use_case = DatabaseViewMappingUseCase::new(state.database_view_mapping_repository.clone());
    let result = use_case
        .get_all_database_view_mappings(
            query.pagination.currentPage,
            query.pagination.itemsPerPage,
            query.database_table_origin_id,
            query.database_table_destiny_id,
            query.entity_type,
            query.data_view_id,
        )
        .await?;

    Ok(Json(result))
}

pub async fn get_database_view_mapping_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<DatabaseViewMappingEntity>>> {
    let use_case = DatabaseViewMappingUseCase::new(state.database_view_mapping_repository.clone());
    let mapping = use_case.get_database_view_mapping_by_id(&id).await?;

    Ok(Json(ApiResponse::success("Mapeamento encontrado", mapping)))
}

pub async fn get_database_view_mappings_by_entity_type(
    State(state): State<AppState>,
    Path(entity_type): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<DatabaseViewMappingEntity>>>> {
    let use_case = DatabaseViewMappingUseCase::new(state.database_view_mapping_repository.clone());
    let mappings = use_case.get_database_view_mappings_by_entity_type(&entity_type).await?;

    Ok(Json(ApiResponse::success("Mapeamentos encontrados", mappings)))
}

pub async fn update_database_view_mapping(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDatabaseViewMappingDto>,
) -> AppResult<Json<ApiResponse<DatabaseViewMappingEntity>>> {
    let use_case = DatabaseViewMappingUseCase::new(state.database_view_mapping_repository.clone());
    let mapping = use_case.update_database_view_mapping(&id, payload).await?;

    Ok(Json(ApiResponse::success(
        "Mapeamento atualizado com sucesso",
        mapping,
    )))
}

pub async fn delete_database_view_mapping(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = DatabaseViewMappingUseCase::new(state.database_view_mapping_repository.clone());
    use_case.delete_database_view_mapping(&id).await?;

    Ok(Json(ApiResponse::success("Mapeamento excluído com sucesso", "Excluído".to_string())))
}

pub async fn get_database_view_mappings_by_view_id(
    State(state): State<AppState>,
    Path(view_id): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<DatabaseViewMappingEntity>>>> {
    let use_case = DatabaseViewMappingUseCase::new(state.database_view_mapping_repository.clone());
    let mappings = use_case.get_database_view_mappings_by_data_view_id(&view_id).await?;

    Ok(Json(ApiResponse::success("Mapeamentos encontrados", mappings)))
}

pub async fn get_database_view_mappings_preview(
    State(state): State<AppState>,
    Path(view_id): Path<String>,
) -> AppResult<Json<ApiResponse<Value>>> {
    let use_case = DatabaseViewMappingUseCase::with_repositories(
        state.database_view_mapping_repository.clone(),
        state.database_view_repository.clone(),
        state.database_configuration_repository.clone(),
        state.database_table_repository.clone(),
        state.database_transformation_repository.clone(),
    );
    let result = use_case.generate_fhir_preview(&view_id).await?;

    Ok(Json(ApiResponse::success(
        "Prévia FHIR gerada com sucesso",
        result
    )))
}
