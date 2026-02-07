use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Bson, Document}};
use chrono::Utc;
use std::sync::Arc;

use crate::domain::entities::DatabaseConfiguration;
use crate::utils::AppError;

#[derive(Clone)]
pub struct DatabaseConfigurationRepository {
    collection: Collection<DatabaseConfiguration>,
}

impl DatabaseConfigurationRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("database_configurations"),
        }
    }
    
    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

impl DatabaseConfigurationRepository {
    pub async fn create(&self, name: String, db_type: String, version: Option<String>, host: String, port: Option<i32>, database: Option<String>, username: Option<String>, password: Option<String>, auth_type: Option<String>, credentials: Option<String>, company_id: String) -> Result<DatabaseConfiguration, AppError> {
        let now = Utc::now();
        
        let config = DatabaseConfiguration {
            id: None,
            name,
            db_type,
            version,
            host,
            port,
            database,
            username,
            password,
            auth_type,
            credentials,
            company_id,
            created_at: now,
            updated_at: now,
        };

        let result = self.collection.insert_one(&config, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut created_config = config;
        created_config.id = result.inserted_id.as_object_id();
        Ok(created_config)
    }

    pub async fn create_with_id(
        &self,
        id: &str,
        name: String,
        db_type: String,
        version: Option<String>,
        host: String,
        port: Option<i32>,
        database: Option<String>,
        username: Option<String>,
        password: Option<String>,
        auth_type: Option<String>,
        credentials: Option<String>,
        company_id: String,
    ) -> Result<DatabaseConfiguration, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        let object_id_bson = object_id.clone();

        let now = Utc::now();

        let config = DatabaseConfiguration {
            id: Some(object_id),
            name,
            db_type,
            version,
            host,
            port,
            database,
            username,
            password,
            auth_type,
            credentials,
            company_id,
            created_at: now,
            updated_at: now,
        };

        let mut doc = mongodb::bson::to_document(&config)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        doc.remove("id");
        doc.remove("_id");
        doc.insert("_id", Bson::ObjectId(object_id_bson));

        let raw_collection: Collection<Document> = self.collection.clone_with_type();
        raw_collection
            .insert_one(doc, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(config)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<DatabaseConfiguration>, AppError> {
        let filter = doc! { "name": name };
        let config = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(config)
    }

    pub async fn find_by_name_and_company_id(
        &self,
        name: &str,
        company_id: &str,
    ) -> Result<Option<DatabaseConfiguration>, AppError> {
        let filter = doc! { "name": name, "company_id": company_id };
        let config = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(config)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<DatabaseConfiguration>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let config = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(config)
    }

    pub async fn update(&self, id: &str, name: Option<String>, db_type: Option<String>, version: Option<String>, host: Option<String>, port: Option<i32>, database: Option<String>, username: Option<String>, password: Option<String>, auth_type: Option<String>, credentials: Option<String>, company_id: Option<String>) -> Result<DatabaseConfiguration, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let mut update_doc = Document::new();
        
        if let Some(name) = name {
            update_doc.insert("name", name);
        }
        if let Some(db_type) = db_type {
            update_doc.insert("db_type", db_type);
        }
        if let Some(version) = version {
            update_doc.insert("version", version);
        }
        if let Some(host) = host {
            update_doc.insert("host", host);
        }
        if let Some(port) = port {
            update_doc.insert("port", port);
        }
        if let Some(database) = database {
            update_doc.insert("database", database);
        }
        if let Some(username) = username {
            update_doc.insert("username", username);
        }
        if let Some(password) = password {
            update_doc.insert("password", password);
        }
        if let Some(auth_type) = auth_type {
            update_doc.insert("auth_type", auth_type);
        }
        if let Some(credentials) = credentials {
            update_doc.insert("credentials", credentials);
        }
        if let Some(company_id) = company_id {
            update_doc.insert("company_id", company_id);
        }
        
        update_doc.insert("updated_at", Utc::now());
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Database configuration not found after update".to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let result = self.collection.delete_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(result.deleted_count > 0)
    }

    pub async fn find_all(&self, page: i64, limit: i64) -> Result<(Vec<DatabaseConfiguration>, i64), AppError> {
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
        
        let mut configs = Vec::new();
        while let Some(config) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            configs.push(config);
        }
        
        let total = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;
        
        Ok((configs, total))
    }

    /// Conta total de configurations de uma empresa
    pub async fn count_by_company_id(&self, company_id: &str) -> Result<u64, AppError> {
        let filter = doc! { "company_id": company_id };
        let count = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(count)
    }
}
