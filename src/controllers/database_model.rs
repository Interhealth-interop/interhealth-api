use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json
};
use serde::Deserialize;

use crate::{
    application::{AppState, DatabaseModelUseCase, MappingValueUseCase},
    core::AuthUser,
    domain::dtos::{DatabaseModelEntity, CreateDatabaseModelDto, UpdateDatabaseModelDto, DatabaseModelValueEntity, UpsertDatabaseModelValueDto, UpdateDatabaseModelValueClientMappingDto},
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

#[derive(Debug, Deserialize)]
pub struct MappingValuesQueryParams {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(rename = "connectionId")]
    pub connection_id: Option<String>,
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
    let include_values = params.values.unwrap_or(false);
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

pub async fn get_database_model_mapping_values(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Query(params): Query<MappingValuesQueryParams>,
) -> AppResult<Json<PaginationResponse<DatabaseModelValueEntity>>> {
    let use_case = MappingValueUseCase::new(state.database_model_value_repository.clone());
    let result = use_case
        .get_mapping_values_by_database_model_and_company(
            &id,
            &auth.company_id,
            params.connection_id.as_deref(),
            params.pagination.currentPage,
            params.pagination.itemsPerPage,
        )
        .await?;

    Ok(Json(result))
}

pub async fn upsert_database_model_mapping_value(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpsertDatabaseModelValueDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseModelValueEntity>>)> {
    state
        .database_model_value_repository
        .upsert_company_client_mapping(
            &id,
            &payload.type_field,
            &payload.code,
            &payload.description,
            &auth.company_id,
            payload.connection_id.as_deref(),
            &payload.source_key,
            &payload.source_description,
        )
        .await?;

    let updated = state
        .database_model_value_repository
        .find_by_owner_type_code(&id, &payload.type_field, &payload.code)
        .await?;

    let company_hex = auth.company_id.clone();
    let clients = updated
        .clients
        .into_iter()
        .filter(|c| c.company_id.to_hex() == company_hex)
        .map(|c| crate::domain::dtos::MappingValueItemEntity {
            source_key: c.source_key,
            source_description: c.source_description,
            status: c.status,
            company_id: c.company_id.to_hex(),
            connection_id: c.connection_id.map(|id| id.to_hex()),
        })
        .collect();

    let entity = DatabaseModelValueEntity {
        id: updated.id.unwrap().to_hex(),
        owner_id: updated.owner_id.to_hex(),
        type_field: updated.type_field,
        code: updated.code,
        description: updated.description,
        clients,
        created_at: updated.created_at.to_rfc3339(),
        updated_at: updated.updated_at.to_rfc3339(),
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Mapping value upserted successfully",
            entity,
        )),
    ))
}

pub async fn update_database_model_value_mapping(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, value_id)): Path<(String, String)>,
    Json(payload): Json<UpdateDatabaseModelValueClientMappingDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseModelValueEntity>>)> {
    state
        .database_model_value_repository
        .upsert_company_client_mapping_by_value_id(
            &id,
            &value_id,
            &auth.company_id,
            payload.connection_id.as_deref(),
            payload.source_key.as_deref(),
            payload.source_description.as_deref(),
            payload.status.as_deref(),
            payload.code.as_deref(),
            payload.description.as_deref(),
        )
        .await?;

    let updated = state
        .database_model_value_repository
        .find_by_id_and_owner(&id, &value_id)
        .await?;

    let company_hex = auth.company_id.clone();
    let clients = updated
        .clients
        .into_iter()
        .filter(|c| c.company_id.to_hex() == company_hex)
        .map(|c| crate::domain::dtos::MappingValueItemEntity {
            source_key: c.source_key,
            source_description: c.source_description,
            status: c.status,
            company_id: c.company_id.to_hex(),
            connection_id: c.connection_id.map(|id| id.to_hex()),
        })
        .collect();

    let entity = DatabaseModelValueEntity {
        id: updated.id.unwrap().to_hex(),
        owner_id: updated.owner_id.to_hex(),
        type_field: updated.type_field,
        code: updated.code,
        description: updated.description,
        clients,
        created_at: updated.created_at.to_rfc3339(),
        updated_at: updated.updated_at.to_rfc3339(),
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Database model value mapping updated successfully",
            entity,
        )),
    ))
}

pub async fn update_database_model_value_connection_mapping(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, value_id, connection_id)): Path<(String, String, String)>,
    Json(payload): Json<UpdateDatabaseModelValueClientMappingDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseModelValueEntity>>)> {
    state
        .database_model_value_repository
        .upsert_company_client_mapping_by_value_id(
            &id,
            &value_id,
            &auth.company_id,
            Some(&connection_id),
            payload.source_key.as_deref(),
            payload.source_description.as_deref(),
            payload.status.as_deref(),
            payload.code.as_deref(),
            payload.description.as_deref(),
        )
        .await?;

    let updated = state
        .database_model_value_repository
        .find_by_id_and_owner(&id, &value_id)
        .await?;

    let company_hex = auth.company_id.clone();
    let connection_hex = connection_id.clone();
    
    // Filter to return only the connection-specific mapping
    let clients = updated
        .clients
        .into_iter()
        .filter(|c| {
            c.company_id.to_hex() == company_hex &&
            c.connection_id.as_ref().map(|id| id.to_hex()) == Some(connection_hex.clone())
        })
        .map(|c| crate::domain::dtos::MappingValueItemEntity {
            source_key: c.source_key,
            source_description: c.source_description,
            status: c.status,
            company_id: c.company_id.to_hex(),
            connection_id: c.connection_id.map(|id| id.to_hex()),
        })
        .collect();

    let entity = DatabaseModelValueEntity {
        id: updated.id.unwrap().to_hex(),
        owner_id: updated.owner_id.to_hex(),
        type_field: updated.type_field,
        code: updated.code,
        description: updated.description,
        clients,
        created_at: updated.created_at.to_rfc3339(),
        updated_at: updated.updated_at.to_rfc3339(),
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Connection mapping value updated successfully",
            entity,
        )),
    ))
}

pub async fn delete_database_model_value(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, value_id)): Path<(String, String)>,
) -> AppResult<(StatusCode, Json<ApiResponse<String>>)> {
    state
        .database_model_value_repository
        .delete_by_id_and_owner_respecting_type(&id, &value_id, &auth.company_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Database model value deleted successfully",
            "Ok".to_string(),
        )),
    ))
}

pub async fn delete_database_model_value_company_mapping(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, value_id, source_key)): Path<(String, String, String)>,
) -> AppResult<(StatusCode, Json<ApiResponse<String>>)> {
    state
        .database_model_value_repository
        .delete_company_client_mapping(&id, &value_id, &source_key, &auth.company_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Database model value mapping deleted successfully",
            "Ok".to_string(),
        )),
    ))
}

pub async fn delete_database_model_value_connection_mapping(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, value_id, connection_id)): Path<(String, String, String)>,
) -> AppResult<(StatusCode, Json<ApiResponse<String>>)> {
    state
        .database_model_value_repository
        .delete_company_client_mapping_by_connection(&id, &value_id, &auth.company_id, &connection_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Database model value connection mapping deleted successfully",
            "Ok".to_string(),
        )),
    ))
}
