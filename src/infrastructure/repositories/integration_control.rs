use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Document}};
use chrono::{DateTime, Utc};
use chrono::Utc as ChronoUtc;
use std::sync::Arc;

use crate::domain::entities::IntegrationControl;
use crate::utils::AppError;

#[derive(Clone)]
pub struct IntegrationControlRepository {
    collection: Collection<IntegrationControl>,
}

impl IntegrationControlRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("integration_controls"),
        }
    }

    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

impl IntegrationControlRepository {
    pub async fn create(
        &self,
        name: String,
        database_view_id: String,
        cron: String,
        date_field: String,
        start_at: Option<DateTime<Utc>>,
        end_at: Option<DateTime<Utc>>,
        control_field: String,
        company_id: String,
    ) -> Result<IntegrationControl, AppError> {
        let now = ChronoUtc::now();

        let control = IntegrationControl {
            id: None,
            name,
            database_view_id,
            cron,
            date_field,
            start_at,
            end_at,
            last_run_at: None,
            control_field,
            company_id,
            created_at: now,
            updated_at: now,
        };

        let result = self
            .collection
            .insert_one(&control, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut created = control;
        created.id = result.inserted_id.as_object_id();
        Ok(created)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<IntegrationControl>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;

        let control = self
            .collection
            .find_one(doc! { "_id": object_id }, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(control)
    }

    pub async fn find_by_database_view_id(
        &self,
        database_view_id: &str,
    ) -> Result<Vec<IntegrationControl>, AppError> {
        use futures::stream::TryStreamExt;

        let mut cursor = self
            .collection
            .find(doc! { "databaseViewId": database_view_id }, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut controls = Vec::new();
        while let Some(c) = cursor
            .try_next()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            controls.push(c);
        }

        Ok(controls)
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        cron: Option<String>,
        date_field: Option<String>,
        start_at: Option<DateTime<Utc>>,
        end_at: Option<DateTime<Utc>>,
        control_field: Option<String>,
    ) -> Result<IntegrationControl, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;

        let mut update_doc = Document::new();

        if let Some(name) = name {
            update_doc.insert("name", name);
        }
        if let Some(cron) = cron {
            update_doc.insert("cron", cron);
        }
        if let Some(date_field) = date_field {
            update_doc.insert("dateField", date_field);
        }
        if let Some(start_at) = start_at {
            update_doc.insert("startAt", start_at);
        }
        if let Some(end_at) = end_at {
            update_doc.insert("endAt", end_at);
        }
        if let Some(control_field) = control_field {
            update_doc.insert("controlField", control_field);
        }

        update_doc.insert("updatedAt", ChronoUtc::now());

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
            .ok_or_else(|| AppError::NotFound("Integration control not found after update".to_string()))
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

    pub async fn delete_by_database_view_id(&self, database_view_id: &str) -> Result<u64, AppError> {
        let result = self
            .collection
            .delete_many(doc! { "databaseViewId": database_view_id }, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.deleted_count)
    }
}
