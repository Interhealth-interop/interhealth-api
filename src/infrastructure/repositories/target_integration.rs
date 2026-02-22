use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Bson, Document}};
use chrono::Utc;
use std::sync::Arc;

use crate::domain::entities::TargetIntegration;
use crate::utils::AppError;

#[derive(Clone)]
pub struct TargetIntegrationRepository {
    collection: Collection<TargetIntegration>,
}

impl TargetIntegrationRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("target_integrations"),
        }
    }

    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

impl TargetIntegrationRepository {
    pub async fn create(
        &self,
        name: String,
        version: Option<String>,
        host: String,
        auth_type: Option<String>,
        credentials: Option<String>,
        database_view_id: String,
        company_id: String,
    ) -> Result<TargetIntegration, AppError> {
        let now = Utc::now();

        let target = TargetIntegration {
            id: None,
            name,
            version,
            host,
            auth_type,
            credentials,
            database_view_id,
            company_id,
            created_at: now,
            updated_at: now,
        };

        let result = self
            .collection
            .insert_one(&target, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut created = target;
        created.id = result.inserted_id.as_object_id();
        Ok(created)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<TargetIntegration>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;

        let target = self
            .collection
            .find_one(doc! { "_id": object_id }, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(target)
    }

    pub async fn find_by_database_view_id(
        &self,
        database_view_id: &str,
    ) -> Result<Option<TargetIntegration>, AppError> {
        let target = self
            .collection
            .find_one(doc! { "databaseViewId": database_view_id }, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(target)
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        version: Option<String>,
        host: Option<String>,
        auth_type: Option<String>,
        credentials: Option<String>,
    ) -> Result<TargetIntegration, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;

        let mut update_doc = Document::new();

        if let Some(name) = name {
            update_doc.insert("name", name);
        }
        if let Some(version) = version {
            update_doc.insert("version", version);
        }
        if let Some(host) = host {
            update_doc.insert("host", host);
        }
        if let Some(auth_type) = auth_type {
            update_doc.insert("authType", auth_type);
        }
        if let Some(credentials) = credentials {
            update_doc.insert("credentials", credentials);
        }

        update_doc.insert("updatedAt", Utc::now());

        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };

        self.collection
            .update_one(filter.clone(), update, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        self.collection
            .find_one(filter, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Target integration not found after update".to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;

        let result = self
            .collection
            .delete_one(doc! { "_id": object_id }, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.deleted_count > 0)
    }
}
