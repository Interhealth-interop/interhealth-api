use std::sync::Arc;

use crate::domain::dtos::{CompanyEntity, CreateCompanyDto, UpdateCompanyDto};
use crate::infrastructure::repositories::CompanyRepository;
use crate::utils::{AppError, AppResult};

pub struct CompanyUseCase {
    repository: Arc<CompanyRepository>,
}

impl CompanyUseCase {
    pub fn new(repository: Arc<CompanyRepository>) -> Self {
        Self { repository }
    }

    fn map_company_to_entity(&self, company: crate::domain::entities::Company) -> AppResult<CompanyEntity> {
        let company_id = company.id.ok_or_else(|| AppError::InternalServerError)?.to_hex();

        Ok(CompanyEntity {
            id: company_id,
            code: company.code,
            name: company.name,
            cnpj: company.cnpj,
            address: company.address,
            number: company.number,
            phone: company.phone,
            email: company.email,
            city: company.city,
            state: company.state,
            zipcode: company.zipcode,
            country: company.country,
            status: company.status,
            created_at: company.created_at.to_rfc3339(),
            updated_at: company.updated_at.to_rfc3339(),
        })
    }

    pub async fn create_company(&self, data: CreateCompanyDto) -> AppResult<CompanyEntity> {
        let company = self.repository.create(crate::infrastructure::repositories::CreateCompanyDto {
            code: data.code,
            name: data.name,
            cnpj: data.cnpj,
            address: data.address,
            number: data.number,
            phone: data.phone,
            email: data.email,
            city: data.city,
            state: data.state,
            zipcode: data.zipcode,
            country: data.country,
        }).await?;

        self.map_company_to_entity(company)
    }

    pub async fn get_all_companies(&self, page: i64, limit: i64, order_field: Option<String>, order_by: Option<String>) -> AppResult<(Vec<CompanyEntity>, i64)> {
        use crate::utils::sort_helper::build_sort_document;
        
        let sort_document = build_sort_document(order_field, order_by);
        let (companies, total) = self.repository.find_all(page, limit, sort_document).await?;
        
        let mut company_entities = Vec::new();
        for company in companies {
            company_entities.push(self.map_company_to_entity(company)?);
        }

        Ok((company_entities, total))
    }

    pub async fn get_company_by_id(&self, id: &str) -> AppResult<CompanyEntity> {
        let company = self.repository.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Empresa não encontrada".to_string()))?;

        self.map_company_to_entity(company)
    }

    pub async fn get_company_by_code(&self, code: &str) -> AppResult<CompanyEntity> {
        let company = self.repository.find_by_code(code).await?
            .ok_or_else(|| AppError::NotFound("Empresa não encontrada".to_string()))?;

        self.map_company_to_entity(company)
    }

    pub async fn get_company_by_cnpj(&self, cnpj: &str) -> AppResult<CompanyEntity> {
        let company = self.repository.find_by_cnpj(cnpj).await?
            .ok_or_else(|| AppError::NotFound("Empresa não encontrada".to_string()))?;

        self.map_company_to_entity(company)
    }

    pub async fn update_company(&self, id: &str, data: UpdateCompanyDto) -> AppResult<CompanyEntity> {
        let company = self.repository.update(id, crate::infrastructure::repositories::UpdateCompanyDto {
            code: data.code,
            name: data.name,
            cnpj: data.cnpj,
            address: data.address,
            number: data.number,
            phone: data.phone,
            email: data.email,
            city: data.city,
            state: data.state,
            zipcode: data.zipcode,
            country: data.country,
            status: data.status,
        }).await?;

        self.map_company_to_entity(company)
    }

    pub async fn change_company_status(&self, id: &str, status: bool) -> AppResult<CompanyEntity> {
        let company = self.repository.change_status(id, status).await?;

        self.map_company_to_entity(company)
    }

    pub async fn delete_company(&self, id: &str) -> AppResult<bool> {
        self.repository.delete(id).await
    }
}
