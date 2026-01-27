use mongodb::{Database, Collection, bson::{doc, DateTime as BsonDateTime}};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use futures::stream::TryStreamExt;
use tracing::info;

use crate::domain::entities::{SyncJobDocument, JobStatus};
use crate::utils::AppError;

#[derive(Clone)]
pub struct SyncJobRepository {
    collection: Collection<SyncJobDocument>,
}

impl SyncJobRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("sync_jobs"),
        }
    }
    
    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

impl SyncJobRepository {
    // ===== CRUD Básico =====
    
    /// Cria um novo job no MongoDB
    pub async fn create(&self, job: &SyncJobDocument) -> Result<SyncJobDocument, AppError> {
        self.collection.insert_one(job, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        // job_id já é o _id, apenas retornamos o job
        Ok(job.clone())
    }
    
    /// Busca job por job_id (que é o _id no MongoDB)
    pub async fn find_by_job_id(&self, job_id: &str) -> Result<Option<SyncJobDocument>, AppError> {
        let filter = doc! { "_id": job_id };
        let job = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(job)
    }
    
    /// Atualiza um job existente
    pub async fn update(&self, job: &SyncJobDocument) -> Result<(), AppError> {
        let filter = doc! { "_id": &job.job_id };
        
        // Serializar o job completo para BSON
        let update_doc = mongodb::bson::to_document(job)
            .map_err(|e| AppError::Database(format!("Failed to serialize job: {}", e)))?;
        
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter, update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    /// Deleta um job por job_id
    pub async fn delete(&self, job_id: &str) -> Result<(), AppError> {
        let filter = doc! { "_id": job_id };
        self.collection.delete_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
    
    // ===== Queries Especializadas =====
    
    /// Busca job ativo (Pending, Running ou Paused) por database_view_id
    /// Garante que só existe 1 job por integração
    pub async fn find_active_by_view_id(&self, database_view_id: &str) -> Result<Option<SyncJobDocument>, AppError> {
        let filter = doc! {
            "database_view_id": database_view_id,
            "status": {
                "$in": ["pending", "running", "paused"]
            }
        };
        
        let job = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(job)
    }
    
    /// Busca QUALQUER job (qualquer status) por database_view_id
    /// Ordena por created_at DESC para pegar o mais recente
    pub async fn find_by_view_id(&self, database_view_id: &str) -> Result<Option<SyncJobDocument>, AppError> {
        use mongodb::options::FindOptions;
        
        let filter = doc! {
            "database_view_id": database_view_id,
        };
        
        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(1)
            .build();
        
        let mut cursor = self.collection.find(filter, options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let job = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(job)
    }
    
    /// Busca jobs por status
    pub async fn find_by_status(&self, status: JobStatus) -> Result<Vec<SyncJobDocument>, AppError> {
        let status_str = match status {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Paused => "paused",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        };
        
        let filter = doc! { "status": status_str };
        let cursor = self.collection.find(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let jobs: Vec<SyncJobDocument> = cursor.try_collect().await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(jobs)
    }
    
    /// Busca jobs por company_id
    pub async fn find_by_company(&self, company_id: &str) -> Result<Vec<SyncJobDocument>, AppError> {
        let filter = doc! { "company_id": company_id };
        let cursor = self.collection.find(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let jobs: Vec<SyncJobDocument> = cursor.try_collect().await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(jobs)
    }
    
    /// Busca jobs pendentes (limit para controlar quantos processar)
    pub async fn find_pending_jobs(&self, limit: i64) -> Result<Vec<SyncJobDocument>, AppError> {
        use mongodb::options::FindOptions;
        
        let filter = doc! { "status": "pending" };
        let options = FindOptions::builder()
            .limit(limit)
            .sort(doc! { "priority": -1, "created_at": 1 })
            .build();
        
        let cursor = self.collection.find(filter, options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let jobs: Vec<SyncJobDocument> = cursor.try_collect().await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(jobs)
    }
    
    // ===== Estatísticas =====
    
    /// Conta jobs por status
    pub async fn count_by_status(&self, status: JobStatus) -> Result<u64, AppError> {
        let status_str = match status {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Paused => "paused",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        };
        
        let filter = doc! { "status": status_str };
        let count = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(count)
    }
    
    /// Conta jobs por company_id e status
    pub async fn count_by_company_and_status(
        &self,
        company_id: &str,
        status: JobStatus,
    ) -> Result<u64, AppError> {
        let status_str = match status {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Paused => "paused",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        };
        
        let filter = doc! {
            "company_id": company_id,
            "status": status_str
        };
        
        let count = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(count)
    }
    
    // ===== Busca com Filtros e Paginação =====
    
    /// Busca jobs com múltiplos filtros e paginação
    pub async fn find_with_filters(
        &self,
        company_id: Option<String>,
        status: Option<JobStatus>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<SyncJobDocument>, AppError> {
        use mongodb::options::FindOptions;
        
        let mut filter = doc! {};
        
        // Filtro por company_id
        if let Some(cid) = company_id {
            filter.insert("company_id", cid);
        }
        
        // Filtro por status
        if let Some(s) = status {
            let status_str = match s {
                JobStatus::Pending => "pending",
                JobStatus::Running => "running",
                JobStatus::Paused => "paused",
                JobStatus::Completed => "completed",
                JobStatus::Failed => "failed",
                JobStatus::Cancelled => "cancelled",
            };
            filter.insert("status", status_str);
        }
        
        // Filtro por data
        if from_date.is_some() || to_date.is_some() {
            let mut date_filter = doc! {};
            
            if let Some(from) = from_date {
                let bson_from = BsonDateTime::from_millis(from.timestamp_millis());
                date_filter.insert("$gte", bson_from);
            }
            
            if let Some(to) = to_date {
                let bson_to = BsonDateTime::from_millis(to.timestamp_millis());
                date_filter.insert("$lte", bson_to);
            }
            
            filter.insert("created_at", date_filter);
        }
        
        // Paginação
        let skip = (page.saturating_sub(1)) * page_size;
        let options = FindOptions::builder()
            .skip(skip)
            .limit(page_size as i64)
            .sort(doc! { "created_at": -1 })
            .build();
        
        // 🔍 DEBUG: Log do filtro e paginação
        info!("🔍 find_with_filters - Filter: {:?}, Page: {}, PageSize: {}, Skip: {}", 
            filter, page, page_size, skip);
        
        let cursor = self.collection.find(filter.clone(), options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let jobs: Vec<SyncJobDocument> = cursor.try_collect().await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        // 🔍 DEBUG: Log do resultado
        info!("🔍 find_with_filters - Found {} jobs", jobs.len());
        
        Ok(jobs)
    }
    
    /// Conta total de jobs com filtros
    pub async fn count_with_filters(
        &self,
        company_id: Option<String>,
        status: Option<JobStatus>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<u64, AppError> {
        let mut filter = doc! {};
        
        if let Some(cid) = company_id {
            filter.insert("company_id", cid);
        }
        
        if let Some(s) = status {
            let status_str = match s {
                JobStatus::Pending => "pending",
                JobStatus::Running => "running",
                JobStatus::Paused => "paused",
                JobStatus::Completed => "completed",
                JobStatus::Failed => "failed",
                JobStatus::Cancelled => "cancelled",
            };
            filter.insert("status", status_str);
        }
        
        if from_date.is_some() || to_date.is_some() {
            let mut date_filter = doc! {};
            
            if let Some(from) = from_date {
                let bson_from = BsonDateTime::from_millis(from.timestamp_millis());
                date_filter.insert("$gte", bson_from);
            }
            
            if let Some(to) = to_date {
                let bson_to = BsonDateTime::from_millis(to.timestamp_millis());
                date_filter.insert("$lte", bson_to);
            }
            
            filter.insert("created_at", date_filter);
        }
        
        let count = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(count)
    }
}
