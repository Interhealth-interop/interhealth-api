// Job structure for synchronization tasks
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a synchronization job that will be processed by workers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    /// Unique identifier for this job
    pub id: String,
    
    /// Database configuration ID (connection to client's Oracle)
    pub database_config_id: String,
    
    /// Database view ID (integration configuration)
    pub database_view_id: String,
    
    /// Entity type (e.g., "encounter", "patient")
    pub entity_type: String,
    
    /// Company ID that owns this sync
    pub company_id: String,
    
    /// Current status of the job
    pub status: JobStatus,
    
    /// Total number of records to process (if known)
    pub total_records: Option<u64>,
    
    /// Number of records successfully processed
    pub processed_records: u64,
    
    /// Number of records that failed to process
    pub failed_records: u64,
    
    /// Current page being processed
    pub current_page: u64,
    
    /// Number of records per page
    pub page_size: u64,
    
    /// Códigos dos itens que falharam (patient_code, encounter_code, etc)
    pub failed_item_codes: Vec<String>,
    
    /// When the job started processing
    pub started_at: Option<DateTime<Utc>>,
    
    /// When the job finished (success or failure)
    pub finished_at: Option<DateTime<Utc>>,
    
    /// When the job was created
    pub created_at: DateTime<Utc>,
}

/// Job status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is waiting in queue
    Pending,
    
    /// Job is currently being processed by a worker
    Running,
    
    /// Job was paused by user (can be resumed)
    Paused,
    
    /// Job completed successfully
    Completed,
    
    /// Job failed with error
    Failed,
    
    /// Job was cancelled by user
    Cancelled,
}

/// Configuration to create a new synchronization job
/// Client only needs to provide the view_id, everything else is fetched automatically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJobConfig {
    /// Database view ID (integration to synchronize)
    #[serde(rename = "databaseViewId")]
    pub database_view_id: String,
    
    /// Optional: number of records per page (default: 100)
    #[serde(rename = "pageSize")]
    pub page_size: Option<u64>,
}

impl SyncJob {
    /// Creates a new synchronization job
    pub fn new(config: SyncJobConfig, entity_type: String, company_id: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            database_config_id: String::new(), // Will be filled by manager
            database_view_id: config.database_view_id,
            entity_type,
            company_id,
            status: JobStatus::Pending,
            total_records: None,
            processed_records: 0,
            failed_records: 0,
            current_page: 0,
            page_size: config.page_size.unwrap_or(100),
            failed_item_codes: Vec::new(),
            started_at: None,
            finished_at: None,
            created_at: Utc::now(),
        }
    }

    /// Marks the job as started
    pub fn start(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Marks the job as completed successfully
    pub fn complete(&mut self) {
        self.status = JobStatus::Completed;
        self.finished_at = Some(Utc::now());
    }

    /// Marca o job como falho
    pub fn fail(&mut self) {
        self.status = JobStatus::Failed;
        self.finished_at = Some(Utc::now());
    }
    
    /// Pausa o job (pode ser resumido depois)
    pub fn pause(&mut self) {
        self.status = JobStatus::Paused;
    }
    
    /// Retoma um job pausado
    pub fn resume(&mut self) {
        self.status = JobStatus::Running;
    }
    
    /// Adiciona um código de item que falhou
    pub fn add_failed_item_code(&mut self, code: String) {
        if !self.failed_item_codes.contains(&code) {
            self.failed_item_codes.push(code);
        }
    }

    /// Updates the progress of the job
    pub fn update_progress(&mut self, page: u64, processed: u64) {
        self.current_page = page;
        self.processed_records += processed;
    }
}