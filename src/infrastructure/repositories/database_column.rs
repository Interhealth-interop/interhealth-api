use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Document}};
use chrono::Utc;
use std::sync::Arc;

use crate::domain::entities::DatabaseColumn;
use crate::utils::AppError;

#[derive(Clone)]
pub struct DatabaseColumnRepository {
    collection: Collection<DatabaseColumn>,
}

impl DatabaseColumnRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("database_columns"),
        }
    }
    
    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

impl DatabaseColumnRepository {
    pub async fn create(
        &self,
        name: String,
        reference: Option<std::collections::HashMap<String, String>>,
        data_type: String,
        is_nullable: bool,
        is_primary_key: bool,
        is_foreign_key: bool,
        description: String,
        max_length: Option<i32>,
        min_length: Option<i32>,
        database_table_id: String,
        company_id: String,
    ) -> Result<DatabaseColumn, AppError> {
        let now = Utc::now();
        
        let column = DatabaseColumn {
            id: None,
            name,
            reference,
            data_type,
            is_nullable,
            is_primary_key,
            is_foreign_key,
            description,
            max_length,
            min_length,
            database_table_id,
            company_id,
            created_at: now,
            updated_at: now,
        };

        let result = self.collection.insert_one(&column, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut created_column = column;
        created_column.id = result.inserted_id.as_object_id();
        Ok(created_column)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<DatabaseColumn>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let column = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(column)
    }

    pub async fn find_by_table_id(&self, table_id: &str) -> Result<Vec<DatabaseColumn>, AppError> {
        use futures::stream::TryStreamExt;
        
        let filter = doc! { "database_table_id": table_id };
        
        let mut cursor = self.collection.find(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut columns = Vec::new();
        while let Some(column) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            columns.push(column);
        }
        
        Ok(columns)
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        reference: Option<std::collections::HashMap<String, String>>,
        data_type: Option<String>,
        is_nullable: Option<bool>,
        is_primary_key: Option<bool>,
        is_foreign_key: Option<bool>,
        description: Option<String>,
        max_length: Option<i32>,
        min_length: Option<i32>,
    ) -> Result<DatabaseColumn, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let mut update_doc = Document::new();
        
        if let Some(name) = name {
            update_doc.insert("name", name);
        }
        if let Some(reference) = reference {
            update_doc.insert("reference", bson::to_bson(&reference).unwrap());
        }
        if let Some(data_type) = data_type {
            update_doc.insert("data_type", data_type);
        }
        if let Some(is_nullable) = is_nullable {
            update_doc.insert("is_nullable", is_nullable);
        }
        if let Some(is_primary_key) = is_primary_key {
            update_doc.insert("is_primary_key", is_primary_key);
        }
        if let Some(is_foreign_key) = is_foreign_key {
            update_doc.insert("is_foreign_key", is_foreign_key);
        }
        if let Some(description) = description {
            update_doc.insert("description", description);
        }
        if let Some(max_length) = max_length {
            update_doc.insert("max_length", max_length);
        }
        if let Some(min_length) = min_length {
            update_doc.insert("min_length", min_length);
        }
        
        update_doc.insert("updated_at", Utc::now());
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Database column not found after update".to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let result = self.collection.delete_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(result.deleted_count > 0)
    }

    pub async fn find_all(&self, page: i64, limit: i64) -> Result<(Vec<DatabaseColumn>, i64), AppError> {
        use mongodb::options::FindOptions;
        use futures::stream::TryStreamExt;
        
        let skip = (page - 1) * limit;
        let filter = doc! {};
        
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(limit)
            .build();
        
        let mut cursor = self.collection.find(filter.clone(), find_options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut columns = Vec::new();
        while let Some(column) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            columns.push(column);
        }
        
        let total = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;
        
        Ok((columns, total))
    }
}
