use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Document}};
use chrono::Utc;
use std::sync::Arc;

use crate::domain::entities::{DatabaseViewMapping, FieldMapping};
use crate::utils::AppError;

#[derive(Clone)]
pub struct DatabaseViewMappingRepository {
    collection: Collection<DatabaseViewMapping>,
}

impl DatabaseViewMappingRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("database_view_mappings"),
        }
    }
    
    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

impl DatabaseViewMappingRepository {
    pub async fn create(
        &self,
        name: String,
        description: String,
        entity_type: String,
        database_table_origin_id: String,
        database_table_destiny_id: String,
        data_view_id: String,
        field_mappings: Vec<FieldMapping>,
        status: String,
    ) -> Result<DatabaseViewMapping, AppError> {
        let now = Utc::now();
        
        let mapping = DatabaseViewMapping {
            id: None,
            name,
            description,
            entity_type,
            database_table_origin_id,
            database_table_destiny_id,
            data_view_id,
            field_mappings,
            status,
            created_at: now,
            updated_at: now,
        };

        let result = self.collection.insert_one(&mapping, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut created_mapping = mapping;
        created_mapping.id = result.inserted_id.as_object_id();
        Ok(created_mapping)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<DatabaseViewMapping>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let mapping = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(mapping)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<DatabaseViewMapping>, AppError> {
        let filter = doc! { "name": name };
        let mapping = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(mapping)
    }

    pub async fn find_by_data_view_id(&self, data_view_id: &str) -> Result<Vec<DatabaseViewMapping>, AppError> {
        use futures::stream::TryStreamExt;
        
        let filter = doc! { "dataViewId": data_view_id };
        
        let mut cursor = self.collection.find(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut mappings = Vec::new();
        while let Some(mapping) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            mappings.push(mapping);
        }
        
        Ok(mappings)
    }

    pub async fn find_by_mapping_type(&self, mapping_type: &str) -> Result<Vec<DatabaseViewMapping>, AppError> {
        use futures::stream::TryStreamExt;
        
        let filter = doc! { "mappingType": mapping_type };
        
        let mut cursor = self.collection.find(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut mappings = Vec::new();
        while let Some(mapping) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            mappings.push(mapping);
        }
        
        Ok(mappings)
    }

    pub async fn find_by_entity_type(&self, entity_type: &str) -> Result<Vec<DatabaseViewMapping>, AppError> {
        use futures::stream::TryStreamExt;
        
        let filter = doc! { "entityType": entity_type };
        
        let mut cursor = self.collection.find(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut mappings = Vec::new();
        while let Some(mapping) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            mappings.push(mapping);
        }
        
        Ok(mappings)
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        entity_type: Option<String>,
        database_table_origin_id: Option<String>,
        database_table_destiny_id: Option<String>,
        data_view_id: Option<String>,
        field_mappings: Option<Vec<FieldMapping>>,
        status: Option<String>,
    ) -> Result<DatabaseViewMapping, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let mut update_doc = Document::new();
        
        if let Some(name) = name {
            update_doc.insert("name", name);
        }
        if let Some(description) = description {
            update_doc.insert("description", description);
        }
        if let Some(entity_type) = entity_type {
            update_doc.insert("entityType", entity_type);
        }
        if let Some(database_table_origin_id) = database_table_origin_id {
            update_doc.insert("databaseTableOriginId", database_table_origin_id);
        }
        if let Some(database_table_destiny_id) = database_table_destiny_id {
            update_doc.insert("databaseTableDestinyId", database_table_destiny_id);
        }
        if let Some(data_view_id) = data_view_id {
            update_doc.insert("dataViewId", data_view_id);
        }
        if let Some(field_mappings) = field_mappings {
            update_doc.insert("fieldMappings", bson::to_bson(&field_mappings).map_err(|e| AppError::Database(e.to_string()))?);
        }
        if let Some(status) = status {
            update_doc.insert("status", status);
        }
        
        update_doc.insert("updated_at", Utc::now());
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Database view mapping not found after update".to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let result = self.collection.delete_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(result.deleted_count > 0)
    }

    pub async fn find_all(
        &self,
        page: i64,
        limit: i64,
        database_table_origin_id: Option<String>,
        database_table_destiny_id: Option<String>,
        entity_type: Option<String>,
        data_view_id: Option<String>,
    ) -> Result<(Vec<DatabaseViewMapping>, i64, i64), AppError> {
        use mongodb::options::FindOptions;
        use futures::stream::TryStreamExt;
        
        let skip = (page - 1) * limit;
        let mut filter = Document::new();
        
        if let Some(origin_id) = database_table_origin_id {
            filter.insert("databaseTableOriginId", origin_id);
        }
        if let Some(destiny_id) = database_table_destiny_id {
            filter.insert("databaseTableDestinyId", destiny_id);
        }
        if let Some(entity) = entity_type {
            filter.insert("entityType", entity);
        }
        if let Some(view_id) = data_view_id {
            filter.insert("dataViewId", view_id);
        }
                
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(limit)
            .build();
        
        let mut cursor = self.collection.find(filter.clone(), find_options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut mappings = Vec::new();
        while let Some(mapping) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            mappings.push(mapping);
        }                
        let total = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;
        
        let pages = if limit > 0 {
            ((total as f64) / (limit as f64)).ceil() as i64
        } else {
            0
        };
        
        Ok((mappings, total, pages))
    }
}
