use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Document}};
use chrono::Utc;
use std::sync::Arc;

use crate::domain::entities::{DatabaseTransformation, ValueMappingItem};
use std::collections::HashMap;
use crate::utils::AppError;

#[derive(Clone)]
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
}

impl DatabaseTransformationRepository {
    pub async fn create(
        &self,
        name: String,
        type_field: String,
        company_id: String,
        value_mappings: HashMap<String, ValueMappingItem>,
    ) -> Result<DatabaseTransformation, AppError> {
        self.create_with_id(None, name, type_field, company_id, value_mappings).await
    }

    pub async fn create_with_id(
        &self,
        id: Option<String>,
        name: String,
        type_field: String,
        company_id: String,
        value_mappings: HashMap<String, ValueMappingItem>,
    ) -> Result<DatabaseTransformation, AppError> {
        let now = Utc::now();
        
        // Parse the provided ID to use as MongoDB _id
        let object_id = if let Some(id_str) = &id {
            Some(ObjectId::parse_str(id_str)
                .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?)
        } else {
            None
        };
        
        let mut transformation = DatabaseTransformation {
            id: object_id,
            name,
            type_field,
            company_id,
            value_mappings,
            created_at: now,
            updated_at: now,
        };

        // Insert the transformation - MongoDB will use the id field as _id
        let insert_result = self.collection.insert_one(&transformation, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        // Set the inserted ID on the transformation if it wasn't provided
        if transformation.id.is_none() {
            transformation.id = insert_result.inserted_id.as_object_id();
        }
        
        Ok(transformation)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<DatabaseTransformation>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let transformation = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(transformation)
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        value_mappings: Option<HashMap<String, ValueMappingItem>>,
    ) -> Result<DatabaseTransformation, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let mut update_doc = Document::new();
        
        if let Some(name) = name {
            update_doc.insert("name", name);
        }
        if let Some(value_mappings) = value_mappings {
            update_doc.insert("value_mappings", bson::to_bson(&value_mappings).map_err(|e| AppError::Database(e.to_string()))?);
        }
        
        update_doc.insert("updated_at", Utc::now());
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Database transformation not found after update".to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let result = self.collection.delete_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(result.deleted_count > 0)
    }

    pub async fn find_all(&self, page: i64, limit: i64, type_filter: Option<String>) -> Result<(Vec<DatabaseTransformation>, i64), AppError> {
        use mongodb::options::FindOptions;
        use futures::stream::TryStreamExt;
        
        let skip = (page - 1) * limit;
        let mut filter = doc! {};
        
        if let Some(type_value) = type_filter {
            filter.insert("type", type_value);
        }
        
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(limit)
            .build();
        
        let mut cursor = self.collection.find(filter.clone(), find_options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut transformations = Vec::new();
        while let Some(transformation) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            transformations.push(transformation);
        }
        
        let total = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;
        
        Ok((transformations, total))
    }
}
