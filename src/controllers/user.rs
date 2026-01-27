use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::application::{AppState, UserUseCase};
use crate::core::AuthUser;
use crate::domain::dtos::{UserEntity, CreateUserDto, UpdateUserDto, ChangeStatusDto, ChangeTypeDto, AssignCompanyDto};
use crate::utils::{AppError, PaginationResponse, PaginationQuery, ApiResponse, AppResult};

pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(mut payload): Json<CreateUserDto>,
) -> AppResult<(StatusCode, Json<ApiResponse<UserEntity>>)> {
    payload.company_id = Some(auth.company_id);
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    let user = use_case.create_user(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Usuário criado com sucesso", user)),
    ))
}

pub async fn get_all_users(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<PaginationResponse<UserEntity>>> {
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    let (users, total) = use_case.get_all_users(pagination.currentPage, pagination.itemsPerPage).await?;
    
    let result = PaginationResponse::new(
        "Usuários recuperados com sucesso",
        users,
        total,
        pagination.currentPage,
        pagination.itemsPerPage,
    );

    Ok(Json(result))
}

pub async fn get_users_by_company(
    State(state): State<AppState>,
    Path(company_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<PaginationResponse<UserEntity>>> {
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    let (users, total) = use_case.get_users_by_company(&company_id, pagination.currentPage, pagination.itemsPerPage).await?;
    
    let result = PaginationResponse::new(
        "Usuários recuperados com sucesso",
        users,
        total,
        pagination.currentPage,
        pagination.itemsPerPage,
    );

    Ok(Json(result))
}

pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<UserEntity>>> {
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    let user = use_case.get_user_by_id(&id).await?;

    Ok(Json(ApiResponse::success("Usuário encontrado", user)))
}

pub async fn get_user_by_email(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> AppResult<Json<ApiResponse<UserEntity>>> {
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    let user = use_case.get_user_by_email(&email).await?;

    Ok(Json(ApiResponse::success("Usuário encontrado", user)))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserDto>,
) -> AppResult<Json<ApiResponse<UserEntity>>> {
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    let user = use_case.update_user(&id, payload).await?;

    Ok(Json(ApiResponse::success("Usuário atualizado com sucesso", user)))
}

pub async fn change_user_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ChangeStatusDto>,
) -> AppResult<Json<ApiResponse<UserEntity>>> {
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    let user = use_case.change_user_status(&id, payload.status).await?;

    Ok(Json(ApiResponse::success("Status do usuário atualizado com sucesso", user)))
}

pub async fn change_user_type(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ChangeTypeDto>,
) -> AppResult<Json<ApiResponse<UserEntity>>> {
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    let user = use_case.change_user_type(&id, &payload.user_type).await?;

    Ok(Json(ApiResponse::success("Tipo do usuário atualizado com sucesso", user)))
}

pub async fn assign_user_to_company(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AssignCompanyDto>,
) -> AppResult<Json<ApiResponse<UserEntity>>> {
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    let user = use_case.assign_user_to_company(&id, &payload.company_id).await?;

    Ok(Json(ApiResponse::success("Usuário atribuído à empresa com sucesso", user)))
}

pub async fn remove_user_from_company(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<UserEntity>>> {
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    let user = use_case.remove_user_from_company(&id).await?;

    Ok(Json(ApiResponse::success("Usuário removido da empresa com sucesso", user)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<String>>> {
    let use_case = UserUseCase::new(
        state.user_repository.clone(),
        state.company_repository.clone(),
    );
    
    use_case.delete_user(&id).await?;

    Ok(Json(ApiResponse::success("Usuário excluído com sucesso", "Deleted".to_string())))
}
