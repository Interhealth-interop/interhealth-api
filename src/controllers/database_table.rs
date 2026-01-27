use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::application::{AppState, DatabaseTableUseCase};
use crate::core::AuthUser;
use crate::domain::dtos::{DatabaseTableEntity, CreateDatabaseTableDto, UpdateDatabaseTableDto};
use crate::utils::{ApiResponse, AppResult, PaginationResponse, PaginationQuery};

#[derive(Debug, Deserialize)]
pub struct DatabaseTableQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(default = "default_false", rename = "includeColumns")]
    pub include_columns: bool,
    #[serde(rename = "tableTypes")]
    pub table_types: Option<String>,
    #[serde(rename = "entityTypes")]
    pub entity_types: Option<String>,
    #[serde(rename = "tableReferences")]
    pub table_references: Option<String>,
}

fn default_false() -> bool {
    false
}

pub async fn create_database_table(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateDatabaseTableDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseTableEntity>>)> {
    let use_case = DatabaseTableUseCase::new(
        state.database_table_repository.clone(),
        state.database_column_repository.clone(),
    );
    let table = use_case.create_database_table(payload, auth.company_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Tabela criada com sucesso", table)),
    ))
}

pub async fn get_all_database_tables(
    State(state): State<AppState>,
    Query(query): Query<DatabaseTableQuery>,
) -> AppResult<Json<PaginationResponse<DatabaseTableEntity>>> {
    let use_case = DatabaseTableUseCase::new(
        state.database_table_repository.clone(),
        state.database_column_repository.clone(),
    );
    
    let table_types = query.table_types
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());
    let entity_types = query.entity_types
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());
    let table_references = query.table_references
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());
    
    let result = use_case
        .get_all_database_tables(
            query.pagination.currentPage,
            query.pagination.itemsPerPage,
            query.include_columns,
            table_types,
            entity_types,
            table_references,
        )
        .await?;

    Ok(Json(result))
}

pub async fn get_database_table_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<DatabaseTableEntity>>> {
    let use_case = DatabaseTableUseCase::new(
        state.database_table_repository.clone(),
        state.database_column_repository.clone(),
    );
    let table = use_case.get_database_table_by_id(&id).await?;

    Ok(Json(ApiResponse::success("Tabela recuperada com sucesso", table)))
}

pub async fn update_database_table(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDatabaseTableDto>,
) -> AppResult<Json<ApiResponse<DatabaseTableEntity>>> {
    let use_case = DatabaseTableUseCase::new(
        state.database_table_repository.clone(),
        state.database_column_repository.clone(),
    );
    let table = use_case.update_database_table(&id, payload).await?;

    Ok(Json(ApiResponse::success(
        "Tabela atualizada com sucesso",
        table,
    )))
}

pub async fn delete_database_table(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = DatabaseTableUseCase::new(
        state.database_table_repository.clone(),
        state.database_column_repository.clone(),
    );
    use_case.delete_database_table(&id).await?;

    Ok(Json(ApiResponse::success("Tabela excluída com sucesso", "Deleted".to_string())))
}

#[derive(Debug, Deserialize)]
pub struct ExternalTableQuery {
    #[serde(rename = "tableName")]
    pub table_name: String,
    #[serde(rename = "tableTypes")]
    pub table_types: Option<String>,
    #[serde(rename = "entityTypes")]
    pub entity_types: Option<String>,
    #[serde(rename = "tableReferences")]
    pub table_references: Option<String>,
}

pub async fn get_database_tables_by_connection_id(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
    Query(query): Query<ExternalTableQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let use_case = DatabaseTableUseCase::new(
        state.database_table_repository.clone(),
        state.database_column_repository.clone(),
    )
    .with_config_repository(state.database_configuration_repository.clone());

    // Parse comma-separated values into vectors
    let table_types = query.table_types.map(|s| s.split(',').map(|v| v.trim().to_string()).collect());
    let entity_types = query.entity_types.map(|s| s.split(',').map(|v| v.trim().to_string()).collect());
    let table_references = query.table_references.map(|s| s.split(',').map(|v| v.trim().to_string()).collect());

    let table_data = use_case
        .get_table_columns_from_external_db(
            &connection_id,
            &query.table_name,
            table_types,
            entity_types,
            table_references,
        )
        .await?;

    Ok(Json(ApiResponse::success(
        "Tabela recuperada com sucesso",
        table_data,
    )))
}
