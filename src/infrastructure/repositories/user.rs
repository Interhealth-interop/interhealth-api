use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Document}};
use chrono::Utc;
use std::sync::Arc;
use sha2::{Sha256, Digest};
use hex;

use crate::domain::entities::User;
use crate::utils::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
    pub password: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub primary_document: Option<String>,
    #[serde(default = "default_status")]
    pub status: bool,
    #[serde(rename = "companyId")]
    pub company_id: Option<String>,
}

fn default_status() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserDto {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    #[serde(rename = "type")]
    pub user_type: Option<String>,
    pub primary_document: Option<String>,
    #[serde(rename = "companyId")]
    pub company_id: Option<String>,
    pub status: Option<bool>,
}

#[derive(Clone)]
pub struct UserRepository {
    collection: Collection<User>,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("users"),
        }
    }
    
    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
    
    fn hash_password(&self, password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    }
    
    pub fn validate_password(&self, plain_password: &str, hashed_password: &str) -> bool {
        let hashed = self.hash_password(plain_password);
        hashed == hashed_password
    }
}

impl UserRepository {
    pub async fn create(&self, user_data: CreateUserDto) -> Result<User, AppError> {
        let now = Utc::now();
        let hashed_password = self.hash_password(&user_data.password);
        
        let user = User {
            id: None,
            name: user_data.name,
            email: user_data.email,
            password: hashed_password,
            status: user_data.status,
            user_type: user_data.user_type,
            primary_document: user_data.primary_document,
            company_id: user_data.company_id,
            created_at: now,
            updated_at: now,
        };

        let result = self.collection.insert_one(&user, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut created_user = user;
        created_user.id = result.inserted_id.as_object_id();
        Ok(created_user)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let user = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let filter = doc! { "email": email };
        let user = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(user)
    }

    pub async fn update(&self, id: &str, user_data: UpdateUserDto) -> Result<User, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let mut update_doc = Document::new();
        
        if let Some(name) = user_data.name {
            update_doc.insert("name", name);
        }
        if let Some(email) = user_data.email {
            update_doc.insert("email", email);
        }
        if let Some(ref password) = user_data.password {
            let hashed = self.hash_password(password);
            update_doc.insert("password", hashed);
        }
        if let Some(user_type) = user_data.user_type {
            update_doc.insert("type", user_type);
        }
        if let Some(primary_document) = user_data.primary_document {
            update_doc.insert("primary_document", primary_document);
        }
        if let Some(company_id) = user_data.company_id {
            update_doc.insert("company_id", company_id);
        }
        if let Some(status) = user_data.status {
            update_doc.insert("status", status);
        }
        
        update_doc.insert("updated_at", Utc::now());
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found after update".to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let update = doc! { 
            "$set": { 
                "status": false,
                "updated_at": Utc::now()
            } 
        };
        
        let result = self.collection.update_one(filter, update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(result.modified_count > 0)
    }

    pub async fn find_all(&self, page: i64, limit: i64, sort_document: Option<Document>) -> Result<(Vec<User>, i64), AppError> {
        use mongodb::options::FindOptions;
        use futures::stream::TryStreamExt;
        
        let skip = (page - 1) * limit;
        let filter = doc! { };
        
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
        
        let mut users = Vec::new();
        while let Some(user) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            users.push(user);
        }
        
        let total = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;
        
        Ok((users, total))
    }
    
    pub async fn find_by_company(&self, company_id: &str, page: i64, limit: i64) -> Result<(Vec<User>, i64), AppError> {
        use mongodb::options::FindOptions;
        use futures::stream::TryStreamExt;
        
        let skip = (page - 1) * limit;
        let filter = doc! { "company_id": company_id, "status": true };
        
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(limit)
            .build();
        
        let mut cursor = self.collection.find(filter.clone(), find_options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut users = Vec::new();
        while let Some(user) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            users.push(user);
        }
        
        let total = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;
        
        Ok((users, total))
    }
    
    pub async fn change_status(&self, id: &str, status: bool) -> Result<User, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let update = doc! { 
            "$set": { 
                "status": status,
                "updated_at": Utc::now()
            } 
        };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found after status change".to_string()))
    }
    
    pub async fn change_type(&self, id: &str, user_type: &str) -> Result<User, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let update = doc! { 
            "$set": { 
                "type": user_type,
                "updated_at": Utc::now()
            } 
        };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found after type change".to_string()))
    }
    
    pub async fn assign_company(&self, id: &str, company_id: &str) -> Result<User, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let update = doc! { 
            "$set": { 
                "company_id": company_id,
                "updated_at": Utc::now()
            } 
        };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found after company assignment".to_string()))
    }
    
    pub async fn remove_company(&self, id: &str) -> Result<User, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let update = doc! { 
            "$unset": { "company_id": "" },
            "$set": { "updated_at": Utc::now() }
        };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("User not found after company removal".to_string()))
    }
}
