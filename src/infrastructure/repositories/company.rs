use mongodb::{Database, Collection, bson::{doc, oid::ObjectId, Bson, Document}};
use chrono::Utc;
use std::sync::Arc;

use crate::domain::entities::Company;
use crate::utils::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCompanyDto {
    pub code: String,
    pub name: String,
    pub cnpj: String,
    pub address: Option<String>,
    pub number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zipcode: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCompanyDto {
    pub code: Option<String>,
    pub name: Option<String>,
    pub cnpj: Option<String>,
    pub address: Option<String>,
    pub number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zipcode: Option<String>,
    pub country: Option<String>,
    pub status: Option<bool>,
}

#[derive(Clone)]
pub struct CompanyRepository {
    collection: Collection<Company>,
}

impl CompanyRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("companies"),
        }
    }
    
    pub fn arc(db: Database) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

impl CompanyRepository {
    pub async fn create(&self, company_data: CreateCompanyDto) -> Result<Company, AppError> {
        let now = Utc::now();
        let company = Company {
            id: None,
            code: company_data.code,
            name: company_data.name,
            cnpj: company_data.cnpj,
            address: company_data.address,
            number: company_data.number,
            phone: company_data.phone,
            email: company_data.email,
            city: company_data.city,
            state: company_data.state,
            zipcode: company_data.zipcode,
            country: company_data.country,
            status: true,
            created_at: now,
            updated_at: now,
        };

        let result = self.collection.insert_one(&company, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut created_company = company;
        created_company.id = result.inserted_id.as_object_id();
        Ok(created_company)
    }

    pub async fn create_with_id(
        &self,
        id: &str,
        company_data: CreateCompanyDto,
    ) -> Result<Company, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        let object_id_bson = object_id.clone();

        let now = Utc::now();
        let company = Company {
            id: Some(object_id),
            code: company_data.code,
            name: company_data.name,
            cnpj: company_data.cnpj,
            address: company_data.address,
            number: company_data.number,
            phone: company_data.phone,
            email: company_data.email,
            city: company_data.city,
            state: company_data.state,
            zipcode: company_data.zipcode,
            country: company_data.country,
            status: true,
            created_at: now,
            updated_at: now,
        };

        let mut doc = mongodb::bson::to_document(&company)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        doc.remove("id");
        doc.remove("_id");
        doc.insert("_id", Bson::ObjectId(object_id_bson));

        let raw_collection: Collection<Document> = self.collection.clone_with_type();
        raw_collection
            .insert_one(doc, None)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(company)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Company>, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let filter = doc! { "_id": object_id };
        let company = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(company)
    }

    pub async fn find_by_code(&self, code: &str) -> Result<Option<Company>, AppError> {
        let filter = doc! { "code": code };
        let company = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(company)
    }

    pub async fn find_by_cnpj(&self, cnpj: &str) -> Result<Option<Company>, AppError> {
        let filter = doc! { "cnpj": cnpj };
        let company = self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(company)
    }

    pub async fn update(&self, id: &str, company_data: UpdateCompanyDto) -> Result<Company, AppError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;
        
        let mut update_doc = Document::new();
        
        if let Some(code) = company_data.code {
            update_doc.insert("code", code);
        }
        if let Some(name) = company_data.name {
            update_doc.insert("name", name);
        }
        if let Some(cnpj) = company_data.cnpj {
            update_doc.insert("cnpj", cnpj);
        }
        if let Some(address) = company_data.address {
            update_doc.insert("address", address);
        }
        if let Some(number) = company_data.number {
            update_doc.insert("number", number);
        }
        if let Some(phone) = company_data.phone {
            update_doc.insert("phone", phone);
        }
        if let Some(email) = company_data.email {
            update_doc.insert("email", email);
        }
        if let Some(city) = company_data.city {
            update_doc.insert("city", city);
        }
        if let Some(state) = company_data.state {
            update_doc.insert("state", state);
        }
        if let Some(zipcode) = company_data.zipcode {
            update_doc.insert("zipcode", zipcode);
        }
        if let Some(country) = company_data.country {
            update_doc.insert("country", country);
        }
        if let Some(status) = company_data.status {
            update_doc.insert("status", status);
        }
        
        update_doc.insert("updated_at", Utc::now());
        
        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter.clone(), update, None).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        self.collection.find_one(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Company not found after update".to_string()))
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

    pub async fn find_all(&self, page: i64, limit: i64) -> Result<(Vec<Company>, i64), AppError> {
        use mongodb::options::FindOptions;
        use futures::stream::TryStreamExt;
        
        let skip = (page - 1) * limit;
        let filter = doc! { "status": true };
        
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(limit)
            .build();
        
        let mut cursor = self.collection.find(filter.clone(), find_options).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let mut companies = Vec::new();
        while let Some(company) = cursor.try_next().await
            .map_err(|e| AppError::Database(e.to_string()))? {
            companies.push(company);
        }
        
        let total = self.collection.count_documents(filter, None).await
            .map_err(|e| AppError::Database(e.to_string()))? as i64;
        
        Ok((companies, total))
    }
    
    pub async fn change_status(&self, id: &str, status: bool) -> Result<Company, AppError> {
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
            .ok_or_else(|| AppError::NotFound("Company not found after status change".to_string()))
    }
}
