use std::sync::Arc;

use chrono::Utc;
use mongodb::{bson::{doc, oid::ObjectId, to_bson}, Database, Collection};

use crate::domain::entities::{DatabaseModelValue, DatabaseModelValueClient};
use crate::utils::AppError;

#[derive(Clone)]
pub struct DatabaseModelValueRepository {
    collection: Collection<DatabaseModelValue>,
}

impl DatabaseModelValueRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("database_model_values"),
        }
    }

    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }

    pub async fn upsert_with_id(&self, entity: DatabaseModelValue) -> Result<(), AppError> {
        let id = entity
            .id
            .ok_or_else(|| AppError::BadRequest("DatabaseModelValue id is required".to_string()))?;

        let now = Utc::now();

        let clients_bson = to_bson(&entity.clients)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let update = doc! {
            "$set": {
                "owner_id": entity.owner_id,
                "type": entity.type_field,
                "code": entity.code,
                "description": entity.description,
                "clients": clients_bson,
                "updated_at": now,
            },
            "$setOnInsert": {
                "created_at": now,
            }
        };

        let options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .update_one(doc! { "_id": id }, update, options)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_by_id_and_owner_respecting_type(
        &self,
        owner_id: &str,
        value_id: &str,
        company_id: &str,
    ) -> Result<(), AppError> {
        let value = self.find_by_id_and_owner(owner_id, value_id).await?;

        if value.type_field == "CUSTOM" {
            return self.delete_by_id_and_owner(owner_id, value_id).await;
        }

        if value.type_field != "DEFAULT" {
            return Err(AppError::BadRequest(
                "Only DEFAULT and CUSTOM are supported".to_string(),
            ));
        }

        let company_object_id = ObjectId::parse_str(company_id)
            .map_err(|_| AppError::BadRequest("Invalid company ID format".to_string()))?;
        let value_object_id = value
            .id
            .ok_or_else(|| AppError::Database("DatabaseModelValue has no id".to_string()))?;

        let now = Utc::now();
        let filter = doc! {
            "_id": value_object_id,
            "owner_id": value.owner_id,
            "clients": { "$elemMatch": { "company_id": company_object_id } }
        };

        let update = doc! {
            "$pull": { "clients": { "company_id": company_object_id } },
            "$set": { "updated_at": now }
        };

        let result = self
            .collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if result.matched_count == 0 {
            return Err(AppError::NotFound(
                "database_model_value company mapping not found".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn upsert_company_client_mapping_by_value_id(
        &self,
        owner_id: &str,
        value_id: &str,
        company_id: &str,
        connection_id: Option<&str>,
        source_key: Option<&str>,
        source_description: Option<&str>,
        status: Option<&str>,
        code: Option<&str>,
        description: Option<&str>,
    ) -> Result<(), AppError> {
        let owner_object_id = ObjectId::parse_str(owner_id)
            .map_err(|_| AppError::BadRequest("Invalid owner_id format".to_string()))?;
        let value_object_id = ObjectId::parse_str(value_id)
            .map_err(|_| AppError::BadRequest("Invalid value_id format".to_string()))?;
        let company_object_id = ObjectId::parse_str(company_id)
            .map_err(|_| AppError::BadRequest("Invalid company ID format".to_string()))?;
        
        let connection_object_id = connection_id
            .map(|id| ObjectId::parse_str(id)
                .map_err(|_| AppError::BadRequest("Invalid connection ID format".to_string())))
            .transpose()?;

        let now = Utc::now();

        let existing = self
            .collection
            .find_one(
                doc! {
                    "_id": value_object_id,
                    "owner_id": owner_object_id,
                },
                None,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("database_model_value not found".to_string()))?;

        // Find existing client based on company_id and optionally connection_id
        let existing_client = existing
            .clients
            .iter()
            .find(|c| {
                c.company_id == company_object_id && 
                c.connection_id == connection_object_id
            });

        if source_key.is_none() || source_description.is_none() {
            if existing_client.is_none() {
                return Err(AppError::BadRequest(
                    "source_key and source_description are required for the first mapping".to_string(),
                ));
            }
        }

        let new_source_key = source_key
            .map(|s| s.to_string())
            .or_else(|| existing_client.map(|c| c.source_key.clone()))
            .ok_or_else(|| {
                AppError::BadRequest("source_key is required".to_string())
            })?;

        let new_source_description = source_description
            .map(|s| s.to_string())
            .or_else(|| existing_client.map(|c| c.source_description.clone()))
            .ok_or_else(|| {
                AppError::BadRequest("source_description is required".to_string())
            })?;

        let source_key_changed = match (existing_client, source_key) {
            (Some(existing_client), Some(incoming_source_key)) => {
                existing_client.source_key != incoming_source_key
            }
            _ => false,
        };

        let code_changed = existing.type_field == "CUSTOM"
            && code.is_some_and(|incoming_code| existing.code != incoming_code);

        let existing_status = existing_client.map(|c| c.status.clone());

        let new_status = if source_key_changed || code_changed {
            "pending".to_string()
        } else {
            status
                .map(|s| s.to_string())
                .or(existing_status)
                .unwrap_or_else(|| "pending".to_string())
        };

        // Enforce uniqueness based on connection_id:
        // - If connection_id is provided: remove only the entry with matching company_id AND connection_id
        // - If connection_id is NOT provided: remove all entries with matching company_id (current behavior)
        let filter_doc = doc! {
            "_id": value_object_id,
            "owner_id": owner_object_id,
        };

        let pull_condition = if let Some(conn_id) = connection_object_id {
            doc! { "company_id": company_object_id, "connection_id": conn_id }
        } else {
            doc! { "company_id": company_object_id }
        };

        let pull_update = doc! {
            "$pull": { "clients": pull_condition },
            "$set": { "updated_at": now }
        };

        self
            .collection
            .update_one(filter_doc.clone(), pull_update, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if existing.type_field == "CUSTOM" {
            let mut set_doc = doc! { "updated_at": now };
            if let Some(code) = code {
                set_doc.insert("code", code);
            }
            if let Some(description) = description {
                set_doc.insert("description", description);
            }

            if set_doc.len() > 1 {
                self
                    .collection
                    .update_one(filter_doc.clone(), doc! { "$set": set_doc }, None)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
        }

        let new_client = DatabaseModelValueClient {
            source_key: new_source_key,
            source_description: new_source_description,
            status: new_status,
            company_id: company_object_id,
            connection_id: connection_object_id,
        };

        let new_client_bson = to_bson(&new_client)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let push_update = doc! {
            "$push": { "clients": new_client_bson },
            "$set": { "updated_at": now }
        };

        self
            .collection
            .update_one(filter_doc, push_update, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn find_by_id_and_owner(
        &self,
        owner_id: &str,
        value_id: &str,
    ) -> Result<DatabaseModelValue, AppError> {
        let owner_object_id = ObjectId::parse_str(owner_id)
            .map_err(|_| AppError::BadRequest("Invalid owner_id format".to_string()))?;
        let value_object_id = ObjectId::parse_str(value_id)
            .map_err(|_| AppError::BadRequest("Invalid value_id format".to_string()))?;

        let value = self
            .collection
            .find_one(
                doc! {
                    "_id": value_object_id,
                    "owner_id": owner_object_id,
                },
                None,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("database_model_value not found".to_string()))?;

        Ok(value)
    }

    pub async fn delete_by_id_and_owner(
        &self,
        owner_id: &str,
        value_id: &str,
    ) -> Result<(), AppError> {
        let owner_object_id = ObjectId::parse_str(owner_id)
            .map_err(|_| AppError::BadRequest("Invalid owner_id format".to_string()))?;
        let value_object_id = ObjectId::parse_str(value_id)
            .map_err(|_| AppError::BadRequest("Invalid value_id format".to_string()))?;

        let result = self
            .collection
            .delete_one(
                doc! {
                    "_id": value_object_id,
                    "owner_id": owner_object_id,
                },
                None,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if result.deleted_count == 0 {
            return Err(AppError::NotFound("database_model_value not found".to_string()));
        }

        Ok(())
    }

        pub async fn find_by_owner_ids(
        &self,
        owner_ids: &[String],
    ) -> Result<Vec<DatabaseModelValue>, AppError> {
        let owner_object_ids: Result<Vec<ObjectId>, _> = owner_ids
            .iter()
            .map(|id| ObjectId::parse_str(id))
            .collect();
        
        let owner_object_ids = owner_object_ids
            .map_err(|_| AppError::BadRequest("Invalid owner_id format".to_string()))?;
        
        let filter = doc! {
            "owner_id": { "$in": owner_object_ids }
        };
        
        let mut cursor = self
            .collection
            .find(filter, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut results = Vec::new();
        while cursor.advance().await.map_err(|e| AppError::Database(e.to_string()))? {
            let value = cursor.deserialize_current()
                .map_err(|e| AppError::Database(e.to_string()))?;
            results.push(value);
        }
        
        Ok(results)
    }

    pub async fn delete_company_client_mapping(
        &self,
        owner_id: &str,
        code: &str,
        source_key: &str,
        company_id: &str,
    ) -> Result<(), AppError> {
        let owner_object_id = ObjectId::parse_str(owner_id)
            .map_err(|_| AppError::BadRequest("Invalid owner_id format".to_string()))?;
        let company_object_id = ObjectId::parse_str(company_id)
            .map_err(|_| AppError::BadRequest("Invalid company ID format".to_string()))?;

        let now = Utc::now();
        let filter = doc! {
            "owner_id": owner_object_id,
            "code": code,
            "clients": { "$elemMatch": { "company_id": company_object_id, "source_key": source_key } }
        };

        let update = doc! {
            "$pull": {
                "clients": {
                    "company_id": company_object_id,
                    "source_key": source_key,
                }
            },
            "$set": { "updated_at": now }
        };

        let result = self
            .collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if result.matched_count == 0 {
            return Err(AppError::NotFound("database_model_value client mapping not found".to_string()));
        }

        Ok(())
    }

    pub async fn find_by_owner_with_default_and_company_custom(
        &self,
        owner_id: &str,
        company_id: &str,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<DatabaseModelValue>, i64), AppError> {
        use futures::stream::TryStreamExt;
        use mongodb::options::FindOptions;

        let owner_object_id = ObjectId::parse_str(owner_id)
            .map_err(|_| AppError::BadRequest("Invalid owner_id format".to_string()))?;
        let company_object_id = ObjectId::parse_str(company_id)
            .map_err(|_| AppError::BadRequest("Invalid company ID format".to_string()))?;

        let skip = (page - 1) * limit;
        let filter = doc! {
            "owner_id": owner_object_id,
            "$or": [
                { "type": "DEFAULT" },
                { "type": "CUSTOM", "clients.company_id": company_object_id },
            ]
        };

        let find_options = FindOptions::builder()
            .skip(skip.max(0) as u64)
            .limit(limit)
            .sort(doc! { "code": 1 })
            .build();

        let mut cursor = self
            .collection
            .find(filter.clone(), find_options)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut values = Vec::new();
        while let Some(item) = cursor
            .try_next()
            .await
            .map_err(|e| AppError::Database(e.to_string()))? {
            values.push(item);
        }

        let total = self
            .collection
            .count_documents(filter, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;

        Ok((values, total))
    }

    pub async fn upsert_company_client_mapping(
        &self,
        owner_id: &str,
        type_field: &str,
        code: &str,
        description: &str,
        company_id: &str,
        connection_id: Option<&str>,
        source_key: &str,
        source_description: &str,
    ) -> Result<(), AppError> {
        let owner_object_id = ObjectId::parse_str(owner_id)
            .map_err(|_| AppError::BadRequest("Invalid owner_id format".to_string()))?;
        let company_object_id = ObjectId::parse_str(company_id)
            .map_err(|_| AppError::BadRequest("Invalid company ID format".to_string()))?;
        
        let connection_object_id = connection_id
            .map(|id| ObjectId::parse_str(id)
                .map_err(|_| AppError::BadRequest("Invalid connection ID format".to_string())))
            .transpose()?;

        let now = Utc::now();

        // Enforce invariant: a company can only have ONE mapping per database_model_value.
        // 1) Ensure the document exists (upsert) and update description.
        let filter_doc = doc! {
            "owner_id": owner_object_id,
            "type": type_field,
            "code": code,
        };

        let upsert_doc = doc! {
            "$set": {
                "description": description,
                "updated_at": now,
            },
            "$setOnInsert": { "created_at": now }
        };

        let options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .update_one(filter_doc.clone(), upsert_doc, options)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 2) Remove any existing mapping for this company_id (and connection_id if provided).
        let pull_condition = if let Some(conn_id) = connection_object_id {
            doc! { "company_id": company_object_id, "connection_id": conn_id }
        } else {
            doc! { "company_id": company_object_id }
        };
        
        let pull_update = doc! {
            "$pull": { "clients": pull_condition },
            "$set": { "updated_at": now }
        };

        self.collection
            .update_one(filter_doc.clone(), pull_update, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 3) Insert a single mapping for this company.
        let new_client = DatabaseModelValueClient {
            source_key: source_key.to_string(),
            source_description: source_description.to_string(),
            status: "pending".to_string(),
            company_id: company_object_id,
            connection_id: connection_object_id,
        };

        let new_client_bson = to_bson(&new_client)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let push_update = doc! {
            "$push": { "clients": new_client_bson },
            "$set": { "updated_at": now }
        };

        self.collection
            .update_one(filter_doc, push_update, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn find_by_owner_type_code(
        &self,
        owner_id: &str,
        type_field: &str,
        code: &str,
    ) -> Result<DatabaseModelValue, AppError> {
        let owner_object_id = ObjectId::parse_str(owner_id)
            .map_err(|_| AppError::BadRequest("Invalid owner_id format".to_string()))?;

        let value = self
            .collection
            .find_one(
                doc! {
                    "owner_id": owner_object_id,
                    "type": type_field,
                    "code": code,
                },
                None,
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("database_model_value not found".to_string()))?;

        Ok(value)
    }

    pub async fn exists_by_id(&self, id: &str) -> Result<bool, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;

        let exists = self
            .collection
            .find_one(doc! { "_id": object_id }, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .is_some();

        Ok(exists)
    }
}
