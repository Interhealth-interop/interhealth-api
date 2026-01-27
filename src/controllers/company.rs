use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::application::{AppState, CompanyUseCase};
use crate::domain::dtos::{CompanyEntity, CreateCompanyDto, UpdateCompanyDto, ChangeCompanyStatusDto};
use crate::utils::{ApiResponse, AppResult, PaginationResponse, PaginationQuery};

pub async fn create_company(
    State(state): State<AppState>,
    Json(payload): Json<CreateCompanyDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<CompanyEntity>>)> {
    let use_case = CompanyUseCase::new(state.company_repository.clone());
    let company = use_case.create_company(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Empresa criada com sucesso", company)),
    ))
}

pub async fn get_all_companies(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<PaginationResponse<CompanyEntity>>> {
    let use_case = CompanyUseCase::new(state.company_repository.clone());
    let (companies, total) = use_case
        .get_all_companies(pagination.currentPage, pagination.itemsPerPage)
        .await?;
    
    let result = PaginationResponse::new(
        "Empresas recuperadas com sucesso",
        companies,
        total,
        pagination.currentPage,
        pagination.itemsPerPage,
    );

    Ok(Json(result))
}

pub async fn get_company_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<CompanyEntity>>> {
    let use_case = CompanyUseCase::new(state.company_repository.clone());
    let company = use_case.get_company_by_id(&id).await?;

    Ok(Json(ApiResponse::success("Empresa encontrada", company)))
}

pub async fn get_company_by_code(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<ApiResponse<CompanyEntity>>> {
    let use_case = CompanyUseCase::new(state.company_repository.clone());
    let company = use_case.get_company_by_code(&code).await?;

    Ok(Json(ApiResponse::success("Empresa encontrada", company)))
}

pub async fn get_company_by_cnpj(
    State(state): State<AppState>,
    Path(cnpj): Path<String>,
) -> AppResult<Json<ApiResponse<CompanyEntity>>> {
    let use_case = CompanyUseCase::new(state.company_repository.clone());
    let company = use_case.get_company_by_cnpj(&cnpj).await?;

    Ok(Json(ApiResponse::success("Empresa encontrada", company)))
}

pub async fn update_company(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCompanyDto>,
) -> AppResult<Json<ApiResponse<CompanyEntity>>> {
    let use_case = CompanyUseCase::new(state.company_repository.clone());
    let company = use_case.update_company(&id, payload).await?;

    Ok(Json(ApiResponse::success("Empresa atualizada com sucesso", company)))
}

pub async fn change_company_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ChangeCompanyStatusDto>,
) -> AppResult<Json<ApiResponse<CompanyEntity>>> {
    let use_case = CompanyUseCase::new(state.company_repository.clone());
    let company = use_case.change_company_status(&id, payload.status).await?;

    Ok(Json(ApiResponse::success("Status da empresa atualizado com sucesso", company)))
}

pub async fn delete_company(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = CompanyUseCase::new(state.company_repository.clone());
    use_case.delete_company(&id).await?;

    Ok(Json(ApiResponse::success("Empresa excluída com sucesso", "Deleted".to_string())))
}
