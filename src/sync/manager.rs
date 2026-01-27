// Sync manager - orchestrates independent job execution with semaphore-based concurrency control
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, error};

use crate::infrastructure::repositories::{
    DatabaseConfigurationRepository, DatabaseViewRepository,
    DatabaseViewMappingRepository, DatabaseTransformationRepository,
    DatabaseTableRepository, SyncJobRepository,
};
use crate::domain::entities::SyncJobDocument;
use super::job::{SyncJob, SyncJobConfig};
use super::status::SyncStatus;
use super::worker::SyncWorker;

/// SyncManager orchestrates independent job execution
/// 
/// New Architecture:
/// - Each JOB spawns its own dedicated Tokio task
/// - Semaphore controls maximum concurrent jobs
/// - Jobs are completely isolated - if one fails, others continue
/// - No shared worker pool or queues
#[derive(Clone)]
pub struct SyncManager {
    /// Semaphore to limit concurrent job execution
    /// Example: max_concurrent = 5 means at most 5 jobs running in parallel
    semaphore: Arc<Semaphore>,
    
    /// Shared status tracker (all jobs write here)
    /// Public para permitir acesso pelo MetricsAggregator
    pub status: Arc<SyncStatus>,
    
    /// Repository references
    sync_job_repo: Arc<SyncJobRepository>,
    db_config_repo: Arc<DatabaseConfigurationRepository>,
    db_view_repo: Arc<DatabaseViewRepository>,
    db_mapping_repo: Arc<DatabaseViewMappingRepository>,
    db_transformation_repo: Arc<DatabaseTransformationRepository>,
    db_table_repo: Arc<DatabaseTableRepository>,
}

impl SyncManager {
    /// Creates a new SyncManager with semaphore-based concurrency control
    /// 
    /// # Arguments
    /// * `max_concurrent_jobs` - Maximum number of jobs running in parallel
    /// * Repository references for database access
    pub fn new(
        max_concurrent_jobs: usize,
        sync_job_repo: Arc<SyncJobRepository>,
        db_config_repo: Arc<DatabaseConfigurationRepository>,
        db_view_repo: Arc<DatabaseViewRepository>,
        db_mapping_repo: Arc<DatabaseViewMappingRepository>,
        db_transformation_repo: Arc<DatabaseTransformationRepository>,
        db_table_repo: Arc<DatabaseTableRepository>,
    ) -> Self {
        info!("🚀 Initializing SyncManager with max {} concurrent jobs", max_concurrent_jobs);
        
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent_jobs)),
            status: Arc::new(SyncStatus::new()),
            sync_job_repo,
            db_config_repo,
            db_view_repo,
            db_mapping_repo,
            db_transformation_repo,
            db_table_repo,
        }
    }

    /// No initialization needed - jobs spawn on-demand
    /// Kept for backward compatibility but does nothing
    pub async fn start(&mut self) {
        info!("✅ SyncManager ready (on-demand job spawning enabled)");
    }

    /// Submits a new synchronization job
    /// Spawns a DEDICATED independent task for this job
    /// 
    /// Example: POST /sync/init { "databaseViewId": "123" }
    /// 
    /// # New Architecture
    /// 1. Validates job configuration
    /// 2. Spawns a dedicated Tokio task for THIS job only
    /// 3. Task acquires semaphore permit (blocks if max concurrent reached)
    /// 4. Processes job in complete isolation
    /// 5. Releases permit when done (success or failure)
    /// 6. If this job fails, other jobs are unaffected
    pub async fn submit_job(
        &self,
        config: SyncJobConfig,
    ) -> Result<SyncJob, crate::utils::AppError> {
        
        // STEP 1: Fetch DatabaseView
        let view = self.db_view_repo
            .find_by_id(&config.database_view_id)
            .await?
            .ok_or_else(|| crate::utils::AppError::NotFound(
                format!("DatabaseView {} not found", config.database_view_id)
            ))?;

        // STEP 2: Fetch DatabaseConfiguration
        let db_config = self.db_config_repo
            .find_by_id(&view.database_configuration_id)
            .await?
            .ok_or_else(|| crate::utils::AppError::NotFound(
                format!("DatabaseConfiguration {} not found", view.database_configuration_id)
            ))?;

        info!(
            "📋 Creating sync job:\n  \
             - View: {} ({})\n  \
             - Entity Type: {}\n  \
             - Company: {}\n  \
             - Database: {}@{}:{}",
            view.name,
            view.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
            view.entity_type,
            view.company_id,
            db_config.username,
            db_config.host,
            db_config.port
        );

        // STEP 3: Create the job with all necessary information
        let mut job = SyncJob::new(
            config.clone(),
            view.entity_type.clone(),
            view.company_id.clone(),
        );

        // Fill in the database_config_id
        job.database_config_id = view.database_configuration_id.clone();

        // Persist job to MongoDB
        let job_doc = SyncJobDocument::from_memory_job(&job);
        self.sync_job_repo.create(&job_doc).await?;
        
        info!("💾 Job {} persisted to MongoDB", job.id);

        // Add to status tracker IMMEDIATELY (so API can query it)
        self.status.add_job(job.clone()).await;

        info!("� Spawning DEDICATED task for job {}", job.id);

        // Clone everything needed for the independent task
        let semaphore = Arc::clone(&self.semaphore);
        let status = Arc::clone(&self.status);
        let sync_job_repo = Arc::clone(&self.sync_job_repo);
        let db_config_repo = Arc::clone(&self.db_config_repo);
        let db_view_repo = Arc::clone(&self.db_view_repo);
        let db_mapping_repo = Arc::clone(&self.db_mapping_repo);
        let db_transformation_repo = Arc::clone(&self.db_transformation_repo);
        let db_table_repo = Arc::clone(&self.db_table_repo);
        let job_clone = job.clone();

        // STEP 4: Spawn DEDICATED task for this job
        // This task is completely independent from all other jobs!
        tokio::spawn(async move {
            // Acquire semaphore permit (waits if max concurrent jobs reached)
            let _permit = semaphore.acquire().await.expect("Semaphore closed");
            
            info!("[JOB-{}] 🔓 Acquired execution slot (semaphore permit)", job_clone.id);

            // Create a dedicated worker just for THIS job
            let worker = SyncWorker::new(
                format!("job-{}", job_clone.id),
                status.clone(),
                sync_job_repo.clone(),
                db_config_repo,
                db_view_repo,
                db_mapping_repo,
                db_transformation_repo,
                db_table_repo,
            );

            // Process this ONE job
            let mut job_mut = job_clone;
            worker.process_single_job(&mut job_mut).await;

            info!("[JOB-{}] 🔒 Released execution slot (task finished)", job_mut.id);
            // Permit is automatically released when _permit is dropped
        });

        info!("✅ Job {} spawned in independent task", job.id);
        
        Ok(job)
    }

    /// Reprocessa um job existente (pausado ou completo)
    /// Similar ao submit_job, mas NÃO cria novo job no MongoDB
    /// Usado para RETOMAR jobs pausados ou REEXECUTAR jobs completos
    pub async fn reprocess_job(&self, mut job: SyncJob) {
        let job_id = job.id.clone(); // Clone ID antes de mover job
        
        info!("🔄 Reprocessando job {}", job_id);

        // Adicionar ao status tracker (para que API possa consultá-lo)
        self.status.add_job(job.clone()).await;

        info!("🚀 Spawning DEDICATED task para reprocessar job {}", job_id);

        // Clone everything needed for the independent task
        let semaphore = Arc::clone(&self.semaphore);
        let status = Arc::clone(&self.status);
        let sync_job_repo = Arc::clone(&self.sync_job_repo);
        let db_config_repo = Arc::clone(&self.db_config_repo);
        let db_view_repo = Arc::clone(&self.db_view_repo);
        let db_mapping_repo = Arc::clone(&self.db_mapping_repo);
        let db_transformation_repo = Arc::clone(&self.db_transformation_repo);
        let db_table_repo = Arc::clone(&self.db_table_repo);

        // Spawn DEDICATED task for this job
        tokio::spawn(async move {
            // Acquire semaphore permit (waits if max concurrent jobs reached)
            let _permit = semaphore.acquire().await.expect("Semaphore closed");
            
            info!("[JOB-{}] 🔓 Acquired execution slot (semaphore permit)", job.id);

            // Create a dedicated worker just for THIS job
            let worker = SyncWorker::new(
                format!("job-{}", job.id),
                status.clone(),
                sync_job_repo.clone(),
                db_config_repo,
                db_view_repo,
                db_mapping_repo,
                db_transformation_repo,
                db_table_repo,
            );

            // Process this ONE job
            worker.process_single_job(&mut job).await;

            info!("[JOB-{}] 🔒 Released execution slot (task finished)", job.id);
            // Permit is automatically released when _permit is dropped
        });

        info!("✅ Job {} reprocessando em task independente", job_id);
    }

    /// Gets the status of a specific job
    pub async fn get_job_status(&self, job_id: &str) -> Option<SyncJob> {
        self.status.get_job(job_id).await
    }

    /// Lists all jobs
    pub async fn list_jobs(&self) -> Vec<SyncJob> {
        self.status.list_jobs().await
    }

    /// Lists jobs for a specific company
    pub async fn list_jobs_by_company(&self, company_id: &str) -> Vec<SyncJob> {
        self.status.list_jobs_by_company(company_id).await
    }

    /// Gets count of currently running jobs
    pub async fn get_running_jobs_count(&self) -> usize {
        self.status.get_running_jobs_count().await
    }
    
    /// Recovers jobs that were running when application was stopped
    /// Should be called during application startup
    pub async fn recover_jobs(&self) -> Result<usize, crate::utils::AppError> {
        use crate::domain::entities::JobStatus as DocStatus;
        
        info!("🔄 Recovering jobs from MongoDB...");
        
        // Find jobs that were running
        let running_jobs = self.sync_job_repo.find_by_status(DocStatus::Running).await?;
        
        let count = running_jobs.len();
        
        if count == 0 {
            info!("✅ No jobs to recover");
            return Ok(0);
        }
        
        info!("📋 Found {} running jobs to recover", count);
        
        for mut job_doc in running_jobs {
            info!("🔄 Recovering job: {} ({})", job_doc.job_id, job_doc.entity_type);
            
            // Reset status to Pending so it will be reprocessed
            job_doc.status = DocStatus::Pending;
            
            // Save updated status
            self.sync_job_repo.update(&job_doc).await?;
            
            // Convert to memory job and resubmit
            // Note: We need to create a SyncJobConfig to resubmit
            let config = SyncJobConfig {
                database_view_id: job_doc.database_view_id.clone(),
                page_size: Some(job_doc.page_size),
            };
            
            // Resubmit the job
            match self.submit_job(config).await {
                Ok(job) => {
                    info!("✅ Job {} recovered and resubmitted", job.id);
                },
                Err(e) => {
                    error!("❌ Failed to recover job {}: {}", job_doc.job_id, e);
                }
            }
        }
        
        info!("✅ Recovery complete: {} jobs resubmitted", count);
        
        Ok(count)
    }
}
