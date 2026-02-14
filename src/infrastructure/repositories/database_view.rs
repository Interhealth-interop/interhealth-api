use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Bson, Document}};
use chrono::Utc;
use std::sync::Arc;

use crate::domain::entities::{DatabaseView, ResourceItem};
use crate::utils::AppError;

#[derive(Clone)]
pub struct DatabaseViewRepository {
    collection: Collection<DatabaseView>,
}

impl DatabaseViewRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("database_views"),
        }
    }
    
    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

impl DatabaseViewRepository {
    pub async fn create(
        &self,
        name: String,
        description: String,
        reference: Option<String>,
        entity_type: String,
        main_resource: Option<String>,
        is_fhir_destination: Option<bool>,
        is_interhealth_destination: Option<bool>,
        database_configuration_id: String,
        company_id: String,
        resources: Option<Vec<ResourceItem>>,
    ) -> Result<DatabaseView, AppError> {
        let now = Utc::now();
        
        let view = DatabaseView {
            id: None,
            name,
            description,
            reference,
            entity_type,
            main_resource,
            is_fhir_destination,
            is_interhealth_destination,
            database_configuration_id,
            company_id,
            status: "pending".to_string(),
            job_id: None,
            resources,
            started_at: None,
            cancelled_at: None,
            created_at: now,
            updated_at: now,
        };

        let result = self.collection.insert_one(&view, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut created_view = view;
        created_view.id = result.inserted_id.as_object_id();
        Ok(created_view)
    }

    pub async fn create_with_id(
        &self,
        id: &str,
        name: String,
        description: String,
        reference: Option<String>,
        entity_type: String,
        main_resource: Option<String>,
        is_fhir_destination: Option<bool>,
        is_interhealth_destination: Option<bool>,
        database_configuration_id: String,
        company_id: String,
        resources: Option<Vec<ResourceItem>>,
    ) -> Result<DatabaseView, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        let object_id_bson = object_id.clone();

        let now = Utc::now();

        let view = DatabaseView {
            id: Some(object_id),
            name,
            description,
            reference,
            entity_type,
            main_resource,
            is_fhir_destination,
            is_interhealth_destination,
            database_configuration_id,
            company_id,
            status: "pending".to_string(),
            job_id: None,
            resources,
            started_at: None,
            cancelled_at: None,
            created_at: now,
            updated_at: now,
        };

        let mut doc = mongodb::bson::to_document(&view)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        doc.remove("id");
        doc.remove("_id");
        doc.insert("_id", Bson::ObjectId(object_id_bson));

        let raw_collection: Collection<Document> = self.collection.clone_with_type();
        raw_collection
            .insert_one(doc, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(view)
    }

    pub async fn find_by_name_and_company_id(
        &self,
        name: &str,
        company_id: &str,
    ) -> Result<Option<DatabaseView>, AppError> {
        let filter = doc! { "name": name, "company_id": company_id };
        let view = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(view)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<DatabaseView>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let view = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(view)
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        reference: Option<String>,
        entity_type: Option<String>,
        main_resource: Option<String>,
        is_fhir_destination: Option<bool>,
        is_interhealth_destination: Option<bool>,
        database_configuration_id: Option<String>,
        status: Option<String>,
        resources: Option<Vec<ResourceItem>>,
    ) -> Result<DatabaseView, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let mut update_doc = Document::new();
        
        if let Some(name) = name {
            update_doc.insert("name", name);
        }
        if let Some(description) = description {
            update_doc.insert("description", description);
        }
        if let Some(reference) = reference {
            update_doc.insert("reference", reference);
        }
        if let Some(entity_type) = entity_type {
            update_doc.insert("entity_type", entity_type);
        }
        if let Some(main_resource) = main_resource {
            update_doc.insert("main_resource", main_resource);
        }
        if let Some(is_fhir_destination) = is_fhir_destination {
            update_doc.insert("is_fhir_destination", is_fhir_destination);
        }
        if let Some(is_interhealth_destination) = is_interhealth_destination {
            update_doc.insert("is_interhealth_destination", is_interhealth_destination);
        }
        if let Some(database_configuration_id) = database_configuration_id {
            update_doc.insert("database_configuration_id", database_configuration_id);
        }
        if let Some(status) = status {
            update_doc.insert("status", status);
        }
        if let Some(resources) = resources {
            let resources_bson = bson::to_bson(&resources)
                .map_err(|e| AppError::Database(format!("Failed to serialize resources: {}", e)))?;
            update_doc.insert("resources", resources_bson);
        }
        
        update_doc.insert("updated_at", Utc::now());
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Database view not found after update".to_string()))
    }

    pub async fn start_integration(&self, id: &str) -> Result<DatabaseView, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let now = Utc::now();
        let mut update_doc = Document::new();
        update_doc.insert("status", "active");
        update_doc.insert("started_at", now);
        update_doc.insert("cancelled_at", mongodb::bson::Bson::Null);
        update_doc.insert("updated_at", now);
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Database view not found after update".to_string()))
    }

    pub async fn cancel_integration(&self, id: &str) -> Result<DatabaseView, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let now = Utc::now();
        let mut update_doc = Document::new();
        update_doc.insert("status", "cancelled");
        update_doc.insert("cancelled_at", now);
        update_doc.insert("updated_at", now);
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Database view not found after update".to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let result = self.collection.delete_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(result.deleted_count > 0)
    }

    pub async fn find_all(&self, page: i64, limit: i64, sort_document: Option<Document>) -> Result<(Vec<DatabaseView>, i64), AppError> {
        use mongodb::options::FindOptions;
        use futures::stream::TryStreamExt;
        
        let skip = (page - 1) * limit;
        let filter = doc! {};
        
        let find_options = if let Some(sort) = sort_document {
            FindOptions::builder()
                .skip(skip as u64)
                .limit(limit)
                .sort(sort)
                .build()
        } else {
            FindOptions::builder()
                .skip(skip as u64)
                .limit(limit)
                .build()
        };
        
        let mut cursor = self.collection.find(filter.clone(), find_options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut views = Vec::new();
        while let Some(view) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            views.push(view);
        }
        
        let total = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;
        
        Ok((views, total))
    }

    /// Conta total de views (integrações) de uma empresa
    pub async fn count_by_company_id(&self, company_id: &str) -> Result<u64, AppError> {
        let filter = doc! { "company_id": company_id };
        let count = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(count)
    }

    /// Atualiza o status da integração baseado no status do job de sincronização
    /// Chamado automaticamente sempre que o status do job muda
    pub async fn update_status_from_job(
        &self,
        database_view_id: &str,
        job_id: &str,
        job_status: &crate::domain::entities::JobStatus,
    ) -> Result<(), AppError> {
        let object_id = ObjectId::parse_str(database_view_id)
            .map_err(|_| AppError::BadRequest("Invalid database_view_id format".to_string()))?;
        
        // Mapear status do job para status da integração
        let integration_status = match job_status {
            crate::domain::entities::JobStatus::Pending => "pending",
            crate::domain::entities::JobStatus::Running => "running",
            crate::domain::entities::JobStatus::Paused => "paused",
            crate::domain::entities::JobStatus::Completed => "completed",
            crate::domain::entities::JobStatus::Failed => "failed",
            crate::domain::entities::JobStatus::Cancelled => "cancelled",
        };
        
        let now = Utc::now();
        let mut update_doc = Document::new();
        update_doc.insert("status", integration_status);
        update_doc.insert("job_id", job_id);
        update_doc.insert("updated_at", now);
        
        // Se o job está começando a rodar pela primeira vez, marcar started_at
        if matches!(job_status, crate::domain::entities::JobStatus::Running) {
            // Verificar se já tem started_at, se não tiver, adicionar
            let filter = doc! { "_id": object_id.clone() };
            if let Ok(Some(view)) = self.collection.find_one(filter, None).await {
                if view.started_at.is_none() {
                    update_doc.insert("started_at", now);
                }
            }
        }
        
        // Se o job foi cancelado, marcar cancelled_at
        if matches!(job_status, crate::domain::entities::JobStatus::Cancelled) {
            update_doc.insert("cancelled_at", now);
        }
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter, update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(())
    }
}
