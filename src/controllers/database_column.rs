use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::application::{AppState, DatabaseColumnUseCase};
use crate::core::AuthUser;
use crate::domain::dtos::{CreateDatabaseColumnDto, UpdateDatabaseColumnDto, DatabaseColumnEntity};
use crate::utils::{ApiResponse, AppResult, PaginationResponse, PaginationQuery};

pub async fn create_database_column(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateDatabaseColumnDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<DatabaseColumnEntity>>)> {
    let use_case = DatabaseColumnUseCase::new(state.database_column_repository.clone());
    let column = use_case.create_database_column(payload, auth.company_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Coluna criada com sucesso", column)),
    ))
}

pub async fn get_all_database_columns(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<PaginationResponse<DatabaseColumnEntity>>> {
    let use_case = DatabaseColumnUseCase::new(state.database_column_repository.clone());
    let result = use_case
        .get_all_database_columns(pagination.currentPage, pagination.itemsPerPage)
        .await?;

    Ok(Json(result))
}

pub async fn get_database_column_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<DatabaseColumnEntity>>> {
    let use_case = DatabaseColumnUseCase::new(state.database_column_repository.clone());
    let column = use_case.get_database_column_by_id(&id).await?;

    Ok(Json(ApiResponse::success("Coluna recuperada com sucesso", column)))
}


pub async fn get_database_columns_by_table_id(
    State(state): State<AppState>,
    Path(table_id): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<DatabaseColumnEntity>>>> {
    let use_case = DatabaseColumnUseCase::new(state.database_column_repository.clone());
    let columns = use_case.get_database_columns_by_table_id(&table_id).await?;

    Ok(Json(ApiResponse::success("Colunas recuperadas com sucesso", columns)))
}

pub async fn update_database_column(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDatabaseColumnDto>,
) -> AppResult<Json<ApiResponse<DatabaseColumnEntity>>> {
    let use_case = DatabaseColumnUseCase::new(state.database_column_repository.clone());
    let column = use_case.update_database_column(&id, payload).await?;

    Ok(Json(ApiResponse::success(
        "Coluna atualizada com sucesso",
        column,
    )))
}

pub async fn delete_database_column(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = DatabaseColumnUseCase::new(state.database_column_repository.clone());
    use_case.delete_database_column(&id).await?;

    Ok(Json(ApiResponse::success("Coluna excluída com sucesso", "Deleted".to_string())))
}
