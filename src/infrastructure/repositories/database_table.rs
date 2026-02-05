use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Document}};
use chrono::Utc;
use std::sync::Arc;

use crate::domain::entities::{DatabaseTable, DatabaseColumn};
use crate::utils::AppError;

#[derive(Clone)]
pub struct DatabaseTableRepository {
    pub collection: Collection<DatabaseTable>,
    column_collection: Collection<DatabaseColumn>,
}

impl DatabaseTableRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("database_tables"),
            column_collection: db.collection("database_columns"),
        }
    }
    
    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

impl DatabaseTableRepository {
    pub async fn create(
        &self,
        name: String,
        description: String,
        table_reference: Option<String>,
        table_type: Option<String>,
        entity_type: String,
        resource: Option<String>,
        company_id: String,
    ) -> Result<DatabaseTable, AppError> {
        let now = Utc::now();
        
        let table = DatabaseTable {
            id: None,
            name,
            description,
            table_reference,
            table_type,
            entity_type,
            resource,
            company_id,
            created_at: now,
            updated_at: now,
        };

        let result = self.collection.insert_one(&table, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut created_table = table;
        created_table.id = result.inserted_id.as_object_id();
        Ok(created_table)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<DatabaseTable>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let table = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(table)
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        table_reference: Option<String>,
        table_type: Option<String>,
        entity_type: Option<String>,
        resource: Option<String>,
    ) -> Result<DatabaseTable, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let mut update_doc = Document::new();
        
        if let Some(name) = name {
            update_doc.insert("name", name);
        }
        if let Some(description) = description {
            update_doc.insert("description", description);
        }
        if let Some(table_reference) = table_reference {
            update_doc.insert("table_reference", table_reference);
        }
        if let Some(table_type) = table_type {
            update_doc.insert("table_type", table_type);
        }
        if let Some(entity_type) = entity_type {
            update_doc.insert("entity_type", entity_type);
        }
        if let Some(resource) = resource {
            update_doc.insert("resource", resource);
        }
        
        update_doc.insert("updated_at", Utc::now());
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Database table not found after update".to_string()))
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
        include_columns: bool,
        table_types: Option<Vec<String>>,
        entity_types: Option<Vec<String>>,
        table_references: Option<Vec<String>>,
    ) -> Result<(Vec<DatabaseTable>, i64, i64), AppError> {
        use mongodb::options::FindOptions;
        use futures::stream::TryStreamExt;
        
        let skip = (page - 1) * limit;
        let mut filter = Document::new();
        
        if let Some(types) = table_types {
            if !types.is_empty() {
                filter.insert("table_type", doc! { "$in": types });
            }
        }
        
        if let Some(types) = entity_types {
            if !types.is_empty() {
                filter.insert("entity_type", doc! { "$in": types });
            }
        }
        
        if let Some(refs) = table_references {
            if !refs.is_empty() {
                filter.insert("table_reference", doc! { "$in": refs });
            }
        }
        
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(limit)
            .build();
        
        let mut cursor = self.collection.find(filter.clone(), find_options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut tables = Vec::new();
        while let Some(table) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            tables.push(table);
        }
        
        let total = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;
        
        let pages = if limit > 0 {
            ((total as f64) / (limit as f64)).ceil() as i64
        } else {
            0
        };
        
        Ok((tables, total, pages))
    }
}
