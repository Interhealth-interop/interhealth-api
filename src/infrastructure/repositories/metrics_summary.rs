use mongodb::{Database, Collection, bson::doc};
use std::sync::Arc;

use crate::domain::entities::metrics_summary::MetricsSummary;
use crate::utils::AppError;

#[derive(Clone)]
pub struct MetricsSummaryRepository {
    collection: Collection<MetricsSummary>,
}

impl MetricsSummaryRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("metrics_summary"),
        }
    }
    
    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
    
    /// Cria ou atualiza o sumário de métricas para uma empresa (upsert)
    /// Sempre há apenas um documento por empresa
    pub async fn upsert(&self, summary: &MetricsSummary) -> Result<MetricsSummary, AppError> {
        use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
        
        let filter = doc! { "company_id": &summary.company_id };
        
        // Serializar summary
        let update_doc = mongodb::bson::to_document(summary)
            .map_err(|e| AppError::Database(format!("Failed to serialize summary: {}", e)))?;
        
        let update = doc! { "$set": update_doc };
        
        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(ReturnDocument::After)
            .build();
        
        let result = self.collection
            .find_one_and_update(filter, update, options)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        result.ok_or_else(|| AppError::Database("Failed to upsert summary".to_string()))
    }
    
    /// Busca o sumário de métricas de uma empresa
    pub async fn find_by_company_id(
        &self,
        company_id: &str,
    ) -> Result<Option<MetricsSummary>, AppError> {
        let filter = doc! { "company_id": company_id };
        
        let summary = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(summary)
    }
    
    /// Deleta o sumário de uma empresa (raramente usado)
    pub async fn delete_by_company_id(
        &self,
        company_id: &str,
    ) -> Result<bool, AppError> {
        let filter = doc! { "company_id": company_id };
        
        let result = self.collection.delete_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(result.deleted_count > 0)
    }
}
