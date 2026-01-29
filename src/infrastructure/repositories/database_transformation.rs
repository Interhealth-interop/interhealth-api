use std::sync::Arc;

use bson::oid::ObjectId;
use chrono::Utc;
use mongodb::{bson::doc, options::{FindOneAndUpdateOptions, ReturnDocument}, Collection, Database};

use crate::domain::entities::DatabaseTransformation;
use crate::utils::AppError;

pub struct DatabaseTransformationRepository {
    collection: Collection<DatabaseTransformation>,
}

impl DatabaseTransformationRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("database_transformations"),
        }
    }

    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<DatabaseTransformation>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;

        let filter = doc! { "_id": object_id };
        self.collection
            .find_one(filter, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn upsert_with_id(
        &self,
        id: Option<String>,
        name: String,
        type_field: String,
        company_id: Option<String>,
        value_mappings: std::collections::HashMap<String, crate::domain::entities::ValueMappingItem>,
    ) -> Result<DatabaseTransformation, AppError> {
        let now = Utc::now();

        let object_id = match id {
            Some(id) => ObjectId::parse_str(&id)
                .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?,
            None => ObjectId::new(),
        };

        let company_object_id = match company_id {
            Some(cid) => Some(ObjectId::parse_str(&cid)
                .map_err(|_| AppError::BadRequest("Invalid company ID format".to_string()))?),
            None => None,
        };

        let filter = doc! { "_id": object_id };
        let update = doc! {
            "$set": {
                "name": name,
                "type": type_field,
                "company_id": company_object_id,
                "value_mappings": bson::to_bson(&value_mappings).map_err(|e| AppError::Database(e.to_string()))?,
                "updated_at": now,
            },
            "$setOnInsert": { "created_at": now }
        };

        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(filter, update, options)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::Database("Failed to upsert transformation".to_string()))
    }
}
