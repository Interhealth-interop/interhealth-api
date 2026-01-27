// Sync controller - REST API endpoints for synchronization
use axum::{
    extract::{State, Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::application::AppState;
use crate::sync::job::{SyncJob, SyncJobConfig};
use crate::domain::entities::{SyncJobDocument, JobStatus};
use crate::utils::{ApiResponse, AppResult, PaginationQuery, PaginationResponse};

/// DTO for starting a synchronization
#[derive(Debug, Deserialize)]
pub struct StartSyncRequest {
    /// Database view ID (integration to synchronize)
    #[serde(rename = "databaseViewId")]
    pub database_view_id: String,
    
    /// Optional: number of records per page (default: 100)
    #[serde(rename = "pageSize")]
    pub page_size: Option<u64>,
}

/// Response after starting a sync
#[derive(Debug, Serialize)]
pub struct StartSyncResponse {
    pub job_id: String,
    pub status: String,
    pub message: String,
}

/// POST /sync/init
/// Inicia/Reinicia uma sincronização
/// 
/// REGRA: Verifica se já existe job cadastrado para a integração (database_view_id)
/// - Se EXISTE job (qualquer status): REINICIA esse job do zero
/// - Se NÃO EXISTE: cria novo job
pub async fn start_sync(
    State(state): State<AppState>,
    Json(payload): Json<StartSyncRequest>,
) -> AppResult<Json<ApiResponse<StartSyncResponse>>> {
    use tracing::info;
    
    // 🔍 PASSO 1: Verificar se já existe QUALQUER job para esta integração
    if let Some(existing_job) = state.sync_job_repository
        .find_by_view_id(&payload.database_view_id)
        .await? 
    {
        info!("🔄 Job existente encontrado ({}), reiniciando...", existing_job.job_id);
        
        // Buscar job da memória ou MongoDB
        let mut job = if let Some(mem_job) = state.sync_manager.get_job_status(&existing_job.job_id).await {
            mem_job
        } else {
            existing_job.to_memory_job()
        };
        
        let old_status = format!("{:?}", job.status);
        
        // 🔄 REINICIAR job do zero
        info!("🔄 Reiniciando job {} (status anterior: {})", job.id, old_status);
        job.current_page = 0;
        job.processed_records = 0;
        job.failed_records = 0;
        job.failed_item_codes.clear();
        job.status = crate::sync::job::JobStatus::Running;
        job.started_at = None;
        job.finished_at = None;
        
        // Aplicar novo page_size se fornecido
        if let Some(new_page_size) = payload.page_size {
            job.page_size = new_page_size;
        }
        
        // Persistir status atualizado no MongoDB
        let job_doc = crate::domain::entities::SyncJobDocument::from_memory_job(&job);
        state.sync_job_repository.update(&job_doc).await?;
        
        // Atualizar status da integração do JOB existente
        let job_status = crate::domain::entities::SyncJobDocument::convert_status(&job.status);
        if let Err(e) = state.database_view_repository.update_status_from_job(&payload.database_view_id, &job.id, &job_status).await {
            info!("Failed to update integration status: {}", e);
        }
        
        // 🚀 REALMENTE PROCESSAR O JOB (spawna task worker)
        state.sync_manager.reprocess_job(job.clone()).await;
        
        let response = StartSyncResponse {
            job_id: job.id.clone(),
            status: "running".to_string(),
            message: format!("Job existente reiniciado do zero! Job ID: {} (status anterior: {})", job.id, old_status),
        };
        
        return Ok(Json(ApiResponse::success("Sincronização Reiniciada", response)));
    }
    
    // 🆕 PASSO 2: Não existe job, criar novo
    info!("🆕 Nenhum job encontrado para database_view_id {}, criando novo...", payload.database_view_id);
    
    let config = SyncJobConfig {
        database_view_id: payload.database_view_id,
        page_size: payload.page_size,
    };

    let mut job = state.sync_manager
        .submit_job(config)
        .await?;

    // Marcar job como Running IMEDIATAMENTE (antes do worker processar)
    job.status = crate::sync::job::JobStatus::Running;
    
    // Atualizar na memória (SyncStatus)
    state.sync_manager.status.add_job(job.clone()).await;
    
    // Persistir status atualizado no MongoDB
    let job_doc = crate::domain::entities::SyncJobDocument::from_memory_job(&job);
    if let Err(e) = state.sync_job_repository.update(&job_doc).await {
        info!("Failed to persist job status: {}", e);
    }
    
    // Atualizar status da integração do JOB novo para "running"
    let job_status = crate::domain::entities::SyncJobDocument::convert_status(&job.status);
    if let Err(e) = state.database_view_repository.update_status_from_job(&job.database_view_id, &job.id, &job_status).await {
        info!("Failed to update integration status: {}", e);
    }

    let response = StartSyncResponse {
        job_id: job.id.clone(),
        status: "running".to_string(),
        message: format!("Sincronização iniciada! Job ID: {}", job.id),
    };

    Ok(Json(ApiResponse::success(
        "Sincronização Iniciada",
        response,
    )))
}

/// GET /sync/status/:job_id
/// Gets the current status of a synchronization job
/// Busca primeiro da memória (tempo real) e depois do MongoDB (fallback)
pub async fn get_sync_status(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<ApiResponse<SyncJob>>> {
    
    // 1️⃣ Tentar buscar da memória primeiro (jobs em execução)
    if let Some(job) = state.sync_manager.get_job_status(&job_id).await {
        return Ok(Json(ApiResponse::success("Job status (tempo real)", job)));
    }
    
    // 2️⃣ Se não encontrou na memória, buscar do MongoDB (jobs completados/antigos)
    match state.sync_job_repository.find_by_job_id(&job_id).await? {
        Some(job_doc) => {
            // Converter documento MongoDB para SyncJob
            let job = job_doc.to_memory_job();
            Ok(Json(ApiResponse::success("Job status (histórico)", job)))
        },
        None => {
            Err(crate::utils::AppError::NotFound(
                format!("Job {} não encontrado", job_id)
            ))
        }
    }
}

/// Query parameters for filtering jobs
#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    #[serde(rename = "companyId")]
    pub company_id: Option<String>,
}

/// Versão resumida do SyncJob (sem failed_item_codes para não inchar a resposta)
#[derive(Debug, Serialize)]
pub struct SyncJobSummary {
    pub id: String,
    pub database_config_id: String,
    pub database_view_id: String,
    pub entity_type: String,
    pub company_id: String,
    pub status: String,
    pub total_records: Option<u64>,
    pub processed_records: u64,
    pub failed_records: u64,
    pub current_page: u64,
    pub page_size: u64,
    pub failed_items_count: usize,  // ✅ Apenas a contagem, não o array completo
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_at: String,
}

/// Versão resumida do SyncJobDocument (sem failed_item_codes)
#[derive(Debug, Serialize)]
pub struct SyncJobDocumentSummary {
    #[serde(rename = "_id")]
    pub job_id: String,
    pub database_view_id: String,
    pub database_config_id: String,
    pub company_id: String,
    pub entity_type: String,
    pub status: String,
    pub total_records: Option<u64>,
    pub processed_records: u64,
    pub failed_records: u64,
    pub current_page: u64,
    pub page_size: u64,
    pub failed_items_count: usize,  // ✅ Apenas a contagem
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

/// Estrutura de estatísticas
#[derive(Debug, Serialize)]
pub struct SyncStatsResponse {
    /// Quantidade de jobs em memória (RAM)
    pub memory_count: usize,
    
    /// Quantidade de jobs no MongoDB
    pub persisted_count: usize,
    
    /// Jobs por status (MongoDB)
    pub running: usize,
    pub pending: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
}

/// GET /sync/stats?companyId=XXX
/// Retorna APENAS estatísticas (sem arrays de jobs)
pub async fn get_sync_stats(
    State(state): State<AppState>,
    Query(query): Query<StatsQuery>,
) -> AppResult<Json<ApiResponse<SyncStatsResponse>>> {
    
    // 1️⃣ Contar jobs em memória
    let memory_count = if let Some(ref company_id) = query.company_id {
        state.sync_manager.list_jobs_by_company(company_id).await.len()
    } else {
        state.sync_manager.list_jobs().await.len()
    };
    
    // 2️⃣ Contar jobs no MongoDB
    let persisted_count = if let Some(ref company_id) = query.company_id {
        state.sync_job_repository
            .find_by_company(company_id)
            .await?
            .len()
    } else {
        state.sync_job_repository
            .count_with_filters(None, None, None, None)
            .await? as usize
    };
    
    // 3️⃣ CALCULAR ESTATÍSTICAS (do MongoDB)
    let running = if let Some(ref company_id) = query.company_id {
        state.sync_job_repository
            .count_by_company_and_status(company_id, JobStatus::Running)
            .await? as usize
    } else {
        state.sync_job_repository
            .count_by_status(JobStatus::Running)
            .await? as usize
    };
    
    let pending = if let Some(ref company_id) = query.company_id {
        state.sync_job_repository
            .count_by_company_and_status(company_id, JobStatus::Pending)
            .await? as usize
    } else {
        state.sync_job_repository
            .count_by_status(JobStatus::Pending)
            .await? as usize
    };
    
    let completed = if let Some(ref company_id) = query.company_id {
        state.sync_job_repository
            .count_by_company_and_status(company_id, JobStatus::Completed)
            .await? as usize
    } else {
        state.sync_job_repository
            .count_by_status(JobStatus::Completed)
            .await? as usize
    };
    
    let failed = if let Some(ref company_id) = query.company_id {
        state.sync_job_repository
            .count_by_company_and_status(company_id, JobStatus::Failed)
            .await? as usize
    } else {
        state.sync_job_repository
            .count_by_status(JobStatus::Failed)
            .await? as usize
    };
    
    let cancelled = if let Some(ref company_id) = query.company_id {
        state.sync_job_repository
            .count_by_company_and_status(company_id, JobStatus::Cancelled)
            .await? as usize
    } else {
        state.sync_job_repository
            .count_by_status(JobStatus::Cancelled)
            .await? as usize
    };
    
    let stats = SyncStatsResponse {
        memory_count,
        persisted_count,
        running,
        pending,
        completed,
        failed,
        cancelled,
    };

    Ok(Json(ApiResponse::success(
        "Estatísticas de sincronização",
        stats,
    )))
}

/// GET /sync/stats/memory?currentPage=1&itemsPerPage=50&companyId=XXX
/// Retorna jobs EM MEMÓRIA (RAM) com paginação
pub async fn get_memory_jobs(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
    Query(query): Query<StatsQuery>,
) -> AppResult<Json<PaginationResponse<SyncJobSummary>>> {
    
    // Buscar TODOS os jobs em memória
    let all_memory_jobs = if let Some(ref company_id) = query.company_id {
        state.sync_manager.list_jobs_by_company(company_id).await
    } else {
        state.sync_manager.list_jobs().await
    };
    
    let total = all_memory_jobs.len() as i64;
    
    // Aplicar paginação manualmente
    let start = ((pagination.currentPage - 1) * pagination.itemsPerPage) as usize;
    let end = (start + pagination.itemsPerPage as usize).min(all_memory_jobs.len());
    
    let paginated_jobs = if start < all_memory_jobs.len() {
        all_memory_jobs[start..end].to_vec()
    } else {
        vec![]
    };
    
    // Converter para summary
    let jobs_summary: Vec<SyncJobSummary> = paginated_jobs.into_iter().map(|job| {
        SyncJobSummary {
            id: job.id,
            database_config_id: job.database_config_id,
            database_view_id: job.database_view_id,
            entity_type: job.entity_type,
            company_id: job.company_id,
            status: format!("{:?}", job.status),
            total_records: job.total_records,
            processed_records: job.processed_records,
            failed_records: job.failed_records,
            current_page: job.current_page,
            page_size: job.page_size,
            failed_items_count: job.failed_item_codes.len(),
            started_at: job.started_at.map(|dt| dt.to_rfc3339()),
            finished_at: job.finished_at.map(|dt| dt.to_rfc3339()),
            created_at: job.created_at.to_rfc3339(),
        }
    }).collect();
    
    let response = PaginationResponse::new(
        "Jobs em memória (RAM)",
        jobs_summary,
        total,
        pagination.currentPage,
        pagination.itemsPerPage,
    );
    
    Ok(Json(response))
}

/// GET /sync/stats/persisted?currentPage=1&itemsPerPage=50&companyId=XXX
/// Retorna jobs PERSISTIDOS no MongoDB com paginação
pub async fn get_persisted_jobs(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
    Query(query): Query<StatsQuery>,
) -> AppResult<Json<PaginationResponse<SyncJobDocumentSummary>>> {
    use tracing::info;
    
    // 🔍 DEBUG: Log dos parâmetros recebidos
    info!("🔍 get_persisted_jobs - Pagination: currentPage={}, itemsPerPage={}", 
        pagination.currentPage, pagination.itemsPerPage);
    info!("🔍 get_persisted_jobs - Query: companyId={:?}", query.company_id);
    
    // Buscar jobs do MongoDB com paginação
    let persisted_jobs = if let Some(ref company_id) = query.company_id {
        state.sync_job_repository
            .find_with_filters(
                Some(company_id.clone()),
                None,
                None,
                None,
                pagination.currentPage as u64,
                pagination.itemsPerPage as u64,
            )
            .await?
    } else {
        state.sync_job_repository
            .find_with_filters(
                None,
                None,
                None,
                None,
                pagination.currentPage as u64,
                pagination.itemsPerPage as u64,
            )
            .await?
    };
    
    // 🔍 DEBUG: Log dos jobs retornados
    info!("🔍 get_persisted_jobs - Received {} jobs from repository", persisted_jobs.len());
    
    // Contar total
    let total = if let Some(ref company_id) = query.company_id {
        state.sync_job_repository
            .count_with_filters(
                Some(company_id.clone()),
                None,
                None,
                None,
            )
            .await? as i64
    } else {
        state.sync_job_repository
            .count_with_filters(None, None, None, None)
            .await? as i64
    };
    
    // 🔍 DEBUG: Log do total
    info!("🔍 get_persisted_jobs - Total count: {}", total);
    
    // Converter para summary
    let jobs_summary: Vec<SyncJobDocumentSummary> = persisted_jobs.into_iter().map(|job| {
        SyncJobDocumentSummary {
            job_id: job.job_id,
            database_view_id: job.database_view_id,
            database_config_id: job.database_config_id,
            company_id: job.company_id,
            entity_type: job.entity_type,
            status: format!("{:?}", job.status),
            total_records: job.total_records,
            processed_records: job.processed_records,
            failed_records: job.failed_records,
            current_page: job.current_page,
            page_size: job.page_size,
            failed_items_count: job.failed_item_codes.len(),
            created_at: job.created_at.to_rfc3339(),
            started_at: job.started_at.map(|dt| dt.to_rfc3339()),
            finished_at: job.finished_at.map(|dt| dt.to_rfc3339()),
        }
    }).collect();
    
    let response = PaginationResponse::new(
        "Jobs persistidos no MongoDB",
        jobs_summary,
        total,
        pagination.currentPage,
        pagination.itemsPerPage,
    );
    
    Ok(Json(response))
}

/// POST /sync/jobs/:job_id/pause
/// Pausa um job em execução
pub async fn pause_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<ApiResponse<SyncJob>>> {
    use tracing::{info, warn};
    
    // 1️⃣ Buscar job (memória ou MongoDB)
    let mut job = if let Some(mem_job) = state.sync_manager.get_job_status(&job_id).await {
        mem_job
    } else {
        // Buscar do MongoDB
        let job_doc = state.sync_job_repository
            .find_by_job_id(&job_id)
            .await?
            .ok_or_else(|| crate::utils::AppError::NotFound(
                format!("Job {} não encontrado", job_id)
            ))?;
        
        job_doc.to_memory_job()
    };
    
    // 2️⃣ Verificar se pode pausar (só Running)
    if job.status != crate::sync::job::JobStatus::Running {
        return Err(crate::utils::AppError::BadRequest(
            format!("Job {} não está em execução (status: {:?})", job_id, job.status)
        ));
    }
    
    info!("⏸️  Pausando job {}", job_id);
    
    // 3️⃣ Pausar job
    job.pause();
    
    // 4️⃣ Atualizar em memória (se ainda estiver lá)
    state.sync_manager.status.add_job(job.clone()).await;
    
    // 5️⃣ Persistir no MongoDB
    let job_doc = crate::domain::entities::SyncJobDocument::from_memory_job(&job);
    state.sync_job_repository.update(&job_doc).await?;
    
    // 6️⃣ Atualizar status da integração
    let job_status = crate::domain::entities::SyncJobDocument::convert_status(&job.status);
    if let Err(e) = state.database_view_repository.update_status_from_job(&job.database_view_id, &job.id, &job_status).await {
        warn!("Failed to update integration status: {}", e);
    }
    
    warn!("⏸️  Job {} pausado com sucesso!", job_id);
    
    Ok(Json(ApiResponse::success("Sincronização Pausada", job)))
}

/// POST /sync/jobs/:job_id/resume
/// Retoma um job pausado
pub async fn resume_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<ApiResponse<SyncJob>>> {
    use tracing::{info, warn};
    
    // 1️⃣ Buscar job (memória ou MongoDB)
    let mut job = if let Some(mem_job) = state.sync_manager.get_job_status(&job_id).await {
        mem_job
    } else {
        // Buscar do MongoDB
        let job_doc = state.sync_job_repository
            .find_by_job_id(&job_id)
            .await?
            .ok_or_else(|| crate::utils::AppError::NotFound(
                format!("Job {} não encontrado", job_id)
            ))?;
        
        job_doc.to_memory_job()
    };
    
    // 2️⃣ Verificar se pode retomar
    if job.status != crate::sync::job::JobStatus::Paused {
        return Err(crate::utils::AppError::BadRequest(
            format!("Job {} não está pausado (status: {:?})", job_id, job.status)
        ));
    }
    
    info!("▶️  Retomando job {}", job_id);
    
    // 3️⃣ Retomar job (muda status para Running)
    job.resume();
    
    // 4️⃣ Persistir status atualizado no MongoDB
    let job_doc = crate::domain::entities::SyncJobDocument::from_memory_job(&job);
    state.sync_job_repository.update(&job_doc).await?;
    
    // 4.5️⃣ Atualizar status da integração
    let job_status = crate::domain::entities::SyncJobDocument::convert_status(&job.status);
    if let Err(e) = state.database_view_repository.update_status_from_job(&job.database_view_id, &job.id, &job_status).await {
        warn!("Failed to update integration status: {}", e);
    }
    
    // 5️⃣ 🚀 REALMENTE PROCESSAR O JOB (spawna task worker)
    state.sync_manager.reprocess_job(job.clone()).await;
    
    warn!("▶️  Job {} retomado e reprocessamento iniciado!", job_id);
    
    Ok(Json(ApiResponse::success("Sincronização Retomada", job)))
}

/// POST /sync/jobs/:job_id/restart
/// Reexecuta um job (Paused, Completed, Failed, qualquer status)
pub async fn restart_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<ApiResponse<SyncJob>>> {
    use tracing::{info, warn};
    
    // 1️⃣ Buscar job (memória ou MongoDB)
    let mut job = if let Some(mem_job) = state.sync_manager.get_job_status(&job_id).await {
        mem_job
    } else {
        // Buscar do MongoDB
        let job_doc = state.sync_job_repository
            .find_by_job_id(&job_id)
            .await?
            .ok_or_else(|| crate::utils::AppError::NotFound(
                format!("Job {} não encontrado", job_id)
            ))?;
        
        job_doc.to_memory_job()
    };
    
    let old_status = format!("{:?}", job.status);
    
    info!("🔄 Reexecutando job {} (status atual: {})", job_id, old_status);
    
    // 2️⃣ Resetar job para reexecutar do início ou continuar de onde parou
    match job.status {
        crate::sync::job::JobStatus::Paused => {
            // Se pausado, continua de onde parou
            info!("📍 Job pausado - continuará da página {}", job.current_page);
            job.resume();
        }
        _ => {
            // Se completed/failed, recomeça do zero
            info!("🔄 Job {} - reexecutando do início", old_status);
            job.current_page = 0;
            job.processed_records = 0;
            job.failed_records = 0;
            job.failed_item_codes.clear();
            job.status = crate::sync::job::JobStatus::Running;
            job.started_at = None;
            job.finished_at = None;
        }
    }
    
    // 3️⃣ Persistir status atualizado no MongoDB
    let job_doc = crate::domain::entities::SyncJobDocument::from_memory_job(&job);
    state.sync_job_repository.update(&job_doc).await?;
    
    // 3.5️⃣ Atualizar status da integração
    let job_status = crate::domain::entities::SyncJobDocument::convert_status(&job.status);
    if let Err(e) = state.database_view_repository.update_status_from_job(&job.database_view_id, &job.id, &job_status).await {
        warn!("Failed to update integration status: {}", e);
    }
    
    // 4️⃣ 🚀 REALMENTE PROCESSAR O JOB (spawna task worker)
    state.sync_manager.reprocess_job(job.clone()).await;
    
    warn!("🔄 Job {} reexecutando! (status anterior: {})", job_id, old_status);
    
    Ok(Json(ApiResponse::success("Resincronizando Integração", job)))
}
