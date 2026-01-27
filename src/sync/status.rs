// Shared status tracking for all synchronization jobs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::job::SyncJob;

pub use super::job::JobStatus;

/// Shared status tracker for all jobs
/// Uses Arc<RwLock<>> to allow multiple workers to read/write safely
#[derive(Clone)]
pub struct SyncStatus {
    /// HashMap storing all jobs by their ID
    /// Arc = Multiple references to the same data
    /// RwLock = Multiple readers OR one writer at a time
    jobs: Arc<RwLock<HashMap<String, SyncJob>>>,
}

impl SyncStatus {
    /// Creates a new status tracker
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds or updates a job in the status tracker
    pub async fn add_job(&self, job: SyncJob) {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id.clone(), job);
    }

    /// Gets a job by its ID
    pub async fn get_job(&self, job_id: &str) -> Option<SyncJob> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned()
    }

    /// Updates a job using a closure
    /// Example: status.update_job("job-123", |job| job.processed_records += 10)
    pub async fn update_job<F>(&self, job_id: &str, update_fn: F)
    where
        F: FnOnce(&mut SyncJob),
    {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(job_id) {
            update_fn(job);
        }
    }

    /// Updates job progress in real-time (for metrics calculation)
    /// This ensures memory state matches MongoDB state
    pub async fn update_job_progress(
        &self,
        job_id: &str,
        processed_records: u64,
        failed_records: u64,
        current_page: u64,
    ) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(job_id) {
            job.processed_records = processed_records;
            job.failed_records = failed_records;
            job.current_page = current_page;
        }
    }

    /// Removes a job from memory
    /// Should be called when job completes/fails to free memory
    pub async fn remove_job(&self, job_id: &str) -> bool {
        let mut jobs = self.jobs.write().await;
        jobs.remove(job_id).is_some()
    }

    /// Lists all jobs
    pub async fn list_jobs(&self) -> Vec<SyncJob> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    /// Lists all jobs for a specific company
    pub async fn list_jobs_by_company(&self, company_id: &str) -> Vec<SyncJob> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| job.company_id == company_id)
            .cloned()
            .collect()
    }

    /// Gets the count of currently running jobs
    pub async fn get_running_jobs_count(&self) -> usize {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| job.status == JobStatus::Running)
            .count()
    }

    /// Gets total count of jobs in memory (for monitoring)
    pub async fn get_total_jobs_count(&self) -> usize {
        let jobs = self.jobs.read().await;
        jobs.len()
    }
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::new()
    }
}
