use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::utils::utils::date_format;

/// Representa um job de sincronização persistido no MongoDB
/// Simplificado - contém apenas campos realmente usados
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncJobDocument {
    /// Job ID (usado como _id no MongoDB)
    #[serde(rename = "_id")]
    pub job_id: String,
    
    /// Database view ID (configuração de integração)
    pub database_view_id: String,
    
    /// Database configuration ID (conexão com banco origem)
    pub database_config_id: String,
    
    /// Company ID que possui este sync
    pub company_id: String,
    
    /// Tipo de entidade (encounter, patient, etc)
    pub entity_type: String,
    
    /// Status do job
    pub status: JobStatus,
    
    /// Total de registros a sincronizar
    pub total_records: Option<u64>,
    
    /// Registros processados com sucesso
    pub processed_records: u64,
    
    /// Registros que falharam
    pub failed_records: u64,
    
    /// Página atual
    pub current_page: u64,
    
    /// Tamanho da página
    pub page_size: u64,
    
    /// Códigos dos itens que falharam (patient_code, encounter_code, etc)
    #[serde(default)]
    pub failed_item_codes: Vec<String>,
    
    /// Data de criação
    #[serde(with = "date_format")]
    pub created_at: DateTime<Utc>,
    
    /// Data de início
    #[serde(default, skip_serializing_if = "Option::is_none", with = "date_format::option")]
    pub started_at: Option<DateTime<Utc>>,
    
    /// Data de conclusão
    #[serde(default, skip_serializing_if = "Option::is_none", with = "date_format::option")]
    pub finished_at: Option<DateTime<Utc>>,
}

/// Status do job de sincronização (simplificado)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job aguardando na fila
    Pending,
    
    /// Job sendo processado
    Running,
    
    /// Job pausado pelo usuário (pode ser resumido)
    Paused,
    
    /// Job completado com sucesso
    Completed,
    
    /// Job falhou
    Failed,
    
    /// Job cancelado pelo usuário
    Cancelled,
}

impl SyncJobDocument {
    /// Cria um novo documento de job a partir do job em memória
    pub fn from_memory_job(job: &crate::sync::job::SyncJob) -> Self {
        Self {
            job_id: job.id.clone(),
            database_view_id: job.database_view_id.clone(),
            database_config_id: job.database_config_id.clone(),
            company_id: job.company_id.clone(),
            entity_type: job.entity_type.clone(),
            status: Self::convert_status(&job.status),
            total_records: job.total_records,
            processed_records: job.processed_records,
            failed_records: job.failed_records,
            current_page: job.current_page,
            page_size: job.page_size,
            failed_item_codes: job.failed_item_codes.clone(),
            created_at: job.created_at,
            started_at: job.started_at,
            finished_at: job.finished_at,
        }
    }
    
    /// Converte status em memória para status persistido
    pub fn convert_status(status: &crate::sync::job::JobStatus) -> JobStatus {
        match status {
            crate::sync::job::JobStatus::Pending => JobStatus::Pending,
            crate::sync::job::JobStatus::Running => JobStatus::Running,
            crate::sync::job::JobStatus::Paused => JobStatus::Paused,
            crate::sync::job::JobStatus::Completed => JobStatus::Completed,
            crate::sync::job::JobStatus::Failed => JobStatus::Failed,
            crate::sync::job::JobStatus::Cancelled => JobStatus::Cancelled,
        }
    }
    
    /// Atualiza campos do documento com dados do job em memória
    pub fn update_from_memory_job(&mut self, job: &crate::sync::job::SyncJob) {
        self.status = Self::convert_status(&job.status);
        self.total_records = job.total_records;
        self.processed_records = job.processed_records;
        self.failed_records = job.failed_records;
        self.current_page = job.current_page;
        self.started_at = job.started_at;
        self.finished_at = job.finished_at;
        self.failed_item_codes = job.failed_item_codes.clone();
    }
    
    /// Converte documento MongoDB de volta para job em memória
    /// Útil para fallback quando job não está mais na memória
    pub fn to_memory_job(&self) -> crate::sync::job::SyncJob {
        crate::sync::job::SyncJob {
            id: self.job_id.clone(),
            database_config_id: self.database_config_id.clone(),
            database_view_id: self.database_view_id.clone(),
            entity_type: self.entity_type.clone(),
            company_id: self.company_id.clone(),
            status: Self::convert_status_back(&self.status),
            total_records: self.total_records,
            processed_records: self.processed_records,
            failed_records: self.failed_records,
            current_page: self.current_page,
            page_size: self.page_size,
            failed_item_codes: self.failed_item_codes.clone(),
            started_at: self.started_at,
            finished_at: self.finished_at,
            created_at: self.created_at,
        }
    }
    
    /// Converte status persistido de volta para status em memória
    fn convert_status_back(status: &JobStatus) -> crate::sync::job::JobStatus {
        match status {
            JobStatus::Pending => crate::sync::job::JobStatus::Pending,
            JobStatus::Running => crate::sync::job::JobStatus::Running,
            JobStatus::Paused => crate::sync::job::JobStatus::Paused,
            JobStatus::Completed => crate::sync::job::JobStatus::Completed,
            JobStatus::Failed => crate::sync::job::JobStatus::Failed,
            JobStatus::Cancelled => crate::sync::job::JobStatus::Cancelled,
        }
    }
}
