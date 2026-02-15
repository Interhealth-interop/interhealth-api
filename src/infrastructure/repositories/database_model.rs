use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Document}};
use chrono::Utc;
use std::sync::Arc;

use crate::domain::entities::{DatabaseModel, ModelValue};
use crate::utils::AppError;

#[derive(Clone)]
pub struct DatabaseModelRepository {
    collection: Collection<DatabaseModel>,
}

impl DatabaseModelRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("database_models"),
        }
    }
    
    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

impl DatabaseModelRepository {
    pub async fn create(
        &self,
        name: String,
        type_field: String,
        description: String,
        reference: Option<String>,
        values: Vec<ModelValue>,
    ) -> Result<DatabaseModel, AppError> {
        self.create_with_id(None, name, type_field, description, reference, values).await
    }

    pub async fn create_with_id(
        &self,
        id: Option<String>,
        name: String,
        type_field: String,
        description: String,
        reference: Option<String>,
        values: Vec<ModelValue>,
    ) -> Result<DatabaseModel, AppError> {
        let now = Utc::now();
        
        let object_id = if let Some(id_str) = &id {
            Some(ObjectId::parse_str(id_str)
                .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?)
        } else {
            None
        };
        
        let mut model = DatabaseModel {
            id: object_id,
            name,
            type_field,
            description,
            reference,
            values,
            created_at: now,
            updated_at: now,
        };

        let insert_result = self.collection.insert_one(&model, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        // Set the inserted ID on the model
        if model.id.is_none() {
            model.id = insert_result.inserted_id.as_object_id();
        }
        
        Ok(model)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<DatabaseModel>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let model = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(model)
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        reference: Option<String>,
        values: Option<Vec<ModelValue>>,
    ) -> Result<DatabaseModel, AppError> {
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
        if let Some(values) = values {
            update_doc.insert("values", bson::to_bson(&values).map_err(|e| AppError::Database(e.to_string()))?);
        }
        
        update_doc.insert("updated_at", Utc::now());
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Database model not found after update".to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let result = self.collection.delete_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(result.deleted_count > 0)
    }

    pub async fn find_all(&self, page: i64, limit: i64, type_filter: Option<String>, sort_document: Option<Document>) -> Result<(Vec<DatabaseModel>, i64), AppError> {
        use mongodb::options::{FindOptions, Collation, CollationStrength};
        use futures::stream::TryStreamExt;
        
        let skip = (page - 1) * limit;
        let mut filter = doc! {};
        
        if let Some(type_value) = type_filter {
            filter.insert("type", type_value);
        }
        
        // Create collation for case-insensitive sorting
        let collation = Collation::builder()
            .locale("en")
            .strength(CollationStrength::Secondary)
            .build();
        
        let find_options = if let Some(sort) = sort_document {
            FindOptions::builder()
                .skip(skip as u64)
                .limit(limit)
                .sort(sort)
                .collation(collation)
                .build()
        } else {
            FindOptions::builder()
                .skip(skip as u64)
                .limit(limit)
                .build()
        };
        
        let mut cursor = self.collection.find(filter.clone(), find_options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut models = Vec::new();
        while let Some(model) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            models.push(model);
        }
        
        let total = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;
        
        Ok((models, total))
    }
}
