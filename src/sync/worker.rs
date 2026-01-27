// Sync worker - processes synchronization jobs independently
// Each worker is created per-job and processes only ONE job before terminating
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, error, warn};
use serde_json::Value;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::infrastructure::adapters::oracledb::OracleConnector;
use crate::infrastructure::repositories::{
    DatabaseConfigurationRepository, DatabaseViewRepository,
    DatabaseViewMappingRepository, DatabaseTransformationRepository,
    DatabaseTableRepository, SyncJobRepository,
};
use crate::domain::entities::SyncJobDocument;
use crate::utils::AppError;
use super::job::SyncJob;
use super::status::SyncStatus;

/// Worker that processes synchronization jobs
/// Each worker runs in its own Tokio task (async thread)
pub struct SyncWorker {
    /// Unique identifier for this worker (e.g., "worker-0", "worker-1")
    worker_id: String,
    
    /// Shared status tracker - all workers write to the same status
    status: Arc<SyncStatus>,
    
    /// Sync job repository for MongoDB persistence
    sync_job_repo: Arc<SyncJobRepository>,
    
    /// Database repositories to fetch configurations
    db_config_repo: Arc<DatabaseConfigurationRepository>,
    db_view_repo: Arc<DatabaseViewRepository>,
    db_mapping_repo: Arc<DatabaseViewMappingRepository>,
    db_transformation_repo: Arc<DatabaseTransformationRepository>,
    db_table_repo: Arc<DatabaseTableRepository>,
}

impl SyncWorker {
    /// Creates a new worker instance
    pub fn new(
        worker_id: String,
        status: Arc<SyncStatus>,
        sync_job_repo: Arc<SyncJobRepository>,
        db_config_repo: Arc<DatabaseConfigurationRepository>,
        db_view_repo: Arc<DatabaseViewRepository>,
        db_mapping_repo: Arc<DatabaseViewMappingRepository>,
        db_transformation_repo: Arc<DatabaseTransformationRepository>,
        db_table_repo: Arc<DatabaseTableRepository>,
    ) -> Self {
        Self {
            worker_id,
            status,
            sync_job_repo,
            db_config_repo,
            db_view_repo,
            db_mapping_repo,
            db_transformation_repo,
            db_table_repo,
        }
    }
    
    /// Verifica se deve simular falha baseado na taxa configurada no .env
    /// 
    /// Lê SIMULATED_FAILURE_RATE do .env e decide aleatoriamente se deve falhar
    /// - Retorna false se a variável não estiver configurada (sem simulação)
    /// - Retorna false se o valor for inválido (< 0.0 ou > 1.0)
    /// - Retorna false se o valor for 0.0 (sem falhas)
    /// - Retorna true/false aleatoriamente baseado na taxa configurada
    fn should_simulate_failure() -> bool {
        std::env::var("SIMULATED_FAILURE_RATE")
            .ok()
            .and_then(|val| val.parse::<f64>().ok())
            .filter(|&rate| rate > 0.0 && rate <= 1.0)
            .map(|rate| rand::random::<f64>() < rate)
            .unwrap_or(false)
    }
    
    /// Extrai o código do item baseado no entity_type
    /// Ex: PATIENT -> patient_code, ENCOUNTER -> encounter_code
    fn extract_item_code(entity_type: &str, record: &serde_json::Value) -> Option<String> {
        let code_field = match entity_type.to_uppercase().as_str() {
            "PATIENT" => "patient_code",
            "ENCOUNTER" => "encounter_code",
            "OBSERVATION" => "observation_code",
            "CONDITION" => "condition_code",
            "PROCEDURE" => "procedure_code",
            "MEDICATION" => "medication_code",
            "ALLERGY" => "allergy_code",
            "LOCATION" => "location_code",
            "ORGANIZATION" => "organization_code",
            "PRACTITIONER" => "practitioner_code",
            _ => return None,
        };
        
        record.get(code_field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Process a SINGLE job and then terminate
    /// Used by dedicated job tasks - no loop, no channel
    /// This ensures complete isolation between jobs
    pub async fn process_single_job(self, job: &mut SyncJob) {
        info!("[{}] 🎯 Processing SINGLE job {}", self.worker_id, job.id);

        // Mark job as running
        job.start();
        self.status.add_job(job.clone()).await;
        
        // Persist status change to MongoDB
        self.persist_job_status(job).await;

        // Process the job (this is where the actual work happens!)
        match self.process_job(job).await {
            Ok(_) => {
                // ⚠️ IMPORTANTE: Verificar se foi pausado antes de marcar como completo!
                if job.status == crate::sync::job::JobStatus::Paused {
                    info!("[{}] ⏸️  Job {} foi pausado durante processamento", self.worker_id, job.id);
                    // Não marcar como complete, já está pausado!
                } else {
                    info!("[{}] ✅ Job {} completed successfully!", self.worker_id, job.id);
                    job.complete();
                }
            }
            Err(e) => {
                error!("[{}] ❌ Job {} failed: {}", self.worker_id, job.id, e);
                job.fail();
            }
        }

        // Persist final status to MongoDB
        self.persist_job_status(job).await;
        
        // LIMPAR MEMÓRIA: Remove job da memória (já está no MongoDB)
        let removed = self.status.remove_job(&job.id).await;
        if removed {
            info!("[{}] 🗑️  Job {} removed from memory (saved in MongoDB)", self.worker_id, job.id);
        }
        
        info!("[{}] 🏁 Task finished for job {}", self.worker_id, job.id);
    }

    /// DEPRECATED: Old worker loop - kept for backward compatibility
    /// New architecture uses process_single_job instead
    #[allow(dead_code)]
    pub async fn run(self, mut rx: tokio::sync::mpsc::Receiver<SyncJob>) {
        info!("[{}] Worker started and waiting for jobs", self.worker_id);

        // Loop: wait for jobs from the channel
        // `while let Some(job) = rx.recv().await` means:
        // "While there are jobs in the channel, get the next one"
        while let Some(mut job) = rx.recv().await {
            info!(
                "[{}] Received job {} for entityType: {}",
                self.worker_id, job.id, job.entity_type
            );

            // Mark job as running
            job.start();
            self.status.add_job(job.clone()).await;

            // Process the job (this is where the actual work happens!)
            match self.process_job(&mut job).await {
                Ok(_) => {
                    info!("[{}] ✅ Job {} completed successfully!", self.worker_id, job.id);
                    job.complete();
                }
                Err(e) => {
                    error!("[{}] ❌ Job {} failed: {}", self.worker_id, job.id, e);
                    job.fail();
                }
            }

            // Persist final status to MongoDB
            self.persist_job_status(&job).await;
            
            // LIMPAR MEMÓRIA: Remove job da memória (já está no MongoDB)
            self.status.remove_job(&job.id).await;
        }

        // If we reach here, the channel was closed (no more jobs)
        info!("[{}] Worker finished (channel closed)", self.worker_id);
    }

    /// Processes a single synchronization job
    /// This is the "brain" of the worker!
    async fn process_job(&self, job: &mut SyncJob) -> Result<(), AppError> {
        // STEP 1: Fetch database view configuration
        let db_view = self.db_view_repo
            .find_by_id(&job.database_view_id)
            .await?
            .ok_or_else(|| AppError::NotFound(
                format!("DatabaseView {} not found", job.database_view_id)
            ))?;
        
        // STEP 2: Fetch database connection configuration
        let db_config = self.db_config_repo
            .find_by_id(&db_view.database_configuration_id)
            .await?
            .ok_or_else(|| AppError::NotFound(
                format!("DatabaseConfiguration {} not found", db_view.database_configuration_id)
            ))?;

        info!(
            "[{}] Connecting to Oracle: {}@{}:{}",
            self.worker_id, db_config.username, db_config.host, db_config.port
        );

        // STEP 3: Connect to client's Oracle database
        let connection_string = format!(
            "oracle://{}:{}@{}:{}/{}",
            db_config.username,
            db_config.password,
            db_config.host,
            db_config.port,
            db_config.database
        );
        
        let oracle_connector = OracleConnector::new(&connection_string).await?;

        // STEP 4: Get table name
        let table_name = format!("{}_INTERHEALTH", db_view.entity_type.to_uppercase());

        // STEP 5: Count total records using connector directly
        info!("[{}] Counting records in table {}", self.worker_id, table_name);
        let total_records = oracle_connector.count_records(&table_name).await?;
        job.total_records = Some(total_records);
        
        // Update job in memory with total_records (importante para cálculo de progresso)
        self.status.update_job(&job.id, |j| {
            j.total_records = Some(total_records);
        }).await;
        
        info!(
            "[{}] Found {} total records to synchronize",
            self.worker_id, total_records
        );

        // If no records, we're done
        if total_records == 0 {
            warn!("[{}] No records to synchronize", self.worker_id);
            return Ok(());
        }

        // STEP 6: Calculate total pages needed
        let total_pages = (total_records as f64 / job.page_size as f64).ceil() as u64;
        
        // 🔄 RETOMADA: Começar da página salva (para jobs pausados/retomados)
        let start_page = job.current_page;
        
        if start_page > 0 {
            info!(
                "[{}] 🔄 RETOMANDO sincronização da página {} até {} (total: {} páginas)",
                self.worker_id, start_page + 1, total_pages, total_pages
            );
        } else {
            info!(
                "[{}] Starting synchronization of {} pages (page_size: {})",
                self.worker_id, total_pages, job.page_size
            );
        }

        // STEP 7: Pagination loop - fetch and process each page
        // Track global record index across all pages
        let mut global_record_index = start_page * job.page_size;
        
        for page in start_page..total_pages {
            // 🔍 VERIFICAR SE JOB FOI PAUSADO
            if let Some(current_job) = self.status.get_job(&job.id).await {
                if current_job.status == crate::sync::job::JobStatus::Paused {
                    warn!(
                        "[{}] ⏸️  Job {} foi PAUSADO! Interrompendo processamento na página {}/{}",
                        self.worker_id,
                        job.id,
                        page + 1,
                        total_pages
                    );
                    
                    // ✅ IMPORTANTE: Atualizar current_page no job antes de salvar!
                    job.current_page = page;
                    job.status = crate::sync::job::JobStatus::Paused;
                    
                    // Salvar estado atual no MongoDB antes de parar
                    self.persist_job_status(job).await;
                    
                    return Ok(()); // Sair do processamento
                }
            }
            
            info!(
                "[{}] Processing page {}/{} of job {}",
                self.worker_id,
                page + 1,
                total_pages,
                job.id
            );

            // Fetch one page of data from Oracle using connector directly
            let records = oracle_connector.fetch_page_data(&table_name, page, job.page_size).await?;
            let records_count = records.len();

            info!(
                "[{}] Fetched {} records from page {}",
                self.worker_id, records_count, page + 1
            );

            // STEP 8: Process each record in this page
            for (idx, record) in records.iter().enumerate() {
                info!(
                    "[{}] Processing record {}/{} from page {} (global index: {})",
                    self.worker_id,
                    idx + 1,
                    records_count,
                    page + 1,
                    global_record_index
                );

                // 🎲 Simulação de falhas para teste de métricas (se configurado no .env)
                if Self::should_simulate_failure() {
                    // Simular falha aleatória
                    // Extrair código do item
                    if let Some(item_code) = Self::extract_item_code(&job.entity_type, record) {
                        job.add_failed_item_code(item_code.clone());
                        error!(
                            "[{}] ❌ SIMULATED FAILURE for record {} (page {}, local {}) - Code: {}",
                            self.worker_id,
                            global_record_index,
                            page + 1,
                            idx + 1,
                            item_code
                        );
                    } else {
                        error!(
                            "[{}] ❌ SIMULATED FAILURE for record {} (page {}, local {}) - Code: N/A",
                            self.worker_id,
                            global_record_index,
                            page + 1,
                            idx + 1
                        );
                    }
                    job.failed_records += 1;
                } else {
                    // Save record to file using GLOBAL index to avoid overwriting
                    match self.save_record_to_file(&job.id, &job.entity_type, global_record_index as usize, record).await {
                        Ok(file_path) => {
                            info!(
                                "[{}] ✅ Record {} (page {}, local {}) saved to: {}",
                                self.worker_id,
                                global_record_index,
                                page + 1,
                                idx + 1,
                                file_path
                            );
                            job.processed_records += 1;  // ✅ Incrementa APENAS no sucesso
                        }
                        Err(e) => {
                            // Extrair código do item que falhou
                            if let Some(item_code) = Self::extract_item_code(&job.entity_type, record) {
                                job.add_failed_item_code(item_code.clone());
                                error!(
                                    "[{}] ❌ Failed to save record {} (page {}, local {}) - Code: {} - Error: {}",
                                    self.worker_id,
                                    global_record_index,
                                    page + 1,
                                    idx + 1,
                                    item_code,
                                    e
                                );
                            } else {
                                error!(
                                    "[{}] ❌ Failed to save record {} (page {}, local {}) - Code: N/A - Error: {}",
                                    self.worker_id,
                                    global_record_index,
                                    page + 1,
                                    idx + 1,
                                    e
                                );
                            }
                            job.failed_records += 1;
                        }
                    }
                }
                
                // Increment global index for next record
                global_record_index += 1;
            }

            // Update job progress (apenas página, processed_records já foi incrementado no sucesso)
            job.current_page = page + 1;
            
            // Update shared status (API can query progress in real-time!)
            self.status.update_job(&job.id, |j| {
                j.current_page = job.current_page;
                j.processed_records = job.processed_records;
                j.failed_records = job.failed_records;
            }).await;

            // 📍 Persist progress to MongoDB after each page
            info!("[{}] 📍 Saving progress at page {}", self.worker_id, page + 1);
            self.persist_job_status(job).await;

            // Small delay between pages to avoid overloading Oracle
            // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }

    /// Saves a record to JSON file in test/ folder
    /// Persists job status to MongoDB
    async fn persist_job_status(&self, job: &SyncJob) {
        use chrono::Utc;
        
        // PASSO 1: Atualizar memória PRIMEIRO (para métricas em tempo real)
        self.status.update_job_progress(
            &job.id,
            job.processed_records,
            job.failed_records,
            job.current_page,
        ).await;
        
        // PASSO 2: Persistir no MongoDB (backup durável)
        // Find existing job document in MongoDB
        match self.sync_job_repo.find_by_job_id(&job.id).await {
            Ok(Some(mut job_doc)) => {
                // Update document with current job status
                job_doc.update_from_memory_job(job);
                
                // Save to MongoDB
                if let Err(e) = self.sync_job_repo.update(&job_doc).await {
                    error!("[{}] Failed to persist job {} to MongoDB: {}", self.worker_id, job.id, e);
                } else {
                    info!("[{}] 💾 Job {} status persisted to MongoDB", self.worker_id, job.id);
                }
                
                // PASSO 3: Atualizar status da integração (DatabaseView) baseado no status do job
                let job_status = SyncJobDocument::convert_status(&job.status);
                if let Err(e) = self.db_view_repo.update_status_from_job(&job.database_view_id, &job.id, &job_status).await {
                    error!("[{}] Failed to update integration status for view {}: {}", self.worker_id, job.database_view_id, e);
                } else {
                    info!("[{}] 🔄 Integration {} status updated to match job status", self.worker_id, job.database_view_id);
                }
            },
            Ok(None) => {
                warn!("[{}] Job {} not found in MongoDB, cannot update", self.worker_id, job.id);
            },
            Err(e) => {
                error!("[{}] Failed to fetch job {} from MongoDB: {}", self.worker_id, job.id, e);
            }
        }
    }
    
    /// For testing purposes instead of sending to FHIR server
    async fn save_record_to_file(
        &self,
        job_id: &str,
        entity_type: &str,
        record_index: usize,
        record: &Value,
    ) -> Result<String, AppError> {
        
        // Create test directory if it doesn't exist
        let test_dir = "sync_data_fhir_test";
        fs::create_dir_all(test_dir).await
            .map_err(|e| AppError::InternalServerError)?;

        // Create job subdirectory
        let job_dir = format!("{}/{}", test_dir, job_id);
        fs::create_dir_all(&job_dir).await
            .map_err(|e| AppError::InternalServerError)?;

        // Generate filename: test/{job_id}/{entity_type}_{index}.json
        let filename = format!(
            "{}/{}_{:04}.json",
            job_dir,
            entity_type.to_lowercase(),
            record_index
        );

        // Serialize record to pretty JSON
        let json_content = serde_json::to_string_pretty(record)
            .map_err(|e| AppError::InternalServerError)?;

        // Write to file
        let mut file = fs::File::create(&filename).await
            .map_err(|e| AppError::InternalServerError)?;
        
        file.write_all(json_content.as_bytes()).await
            .map_err(|e| AppError::InternalServerError)?;

        Ok(filename)
    }
}
