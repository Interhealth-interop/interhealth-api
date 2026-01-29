use mongodb::Database;
use std::sync::Arc;

use crate::core::JwtService;
use crate::infrastructure::repositories::{
    CompanyRepository, UserRepository, DatabaseConfigurationRepository,
    DatabaseColumnRepository, DatabaseTableRepository, DatabaseViewRepository,
    DatabaseViewMappingRepository, DatabaseTransformationRepository, SyncJobRepository,
    MetricsSummaryRepository, DatabaseModelRepository, DatabaseModelValueRepository
};
use crate::application::usecases::MetricsUseCase;
use crate::sync::SyncManager;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub jwt_service: Arc<JwtService>,
    pub company_repository: Arc<CompanyRepository>,
    pub user_repository: Arc<UserRepository>,
    pub database_configuration_repository: Arc<DatabaseConfigurationRepository>,
    pub database_column_repository: Arc<DatabaseColumnRepository>,
    pub database_table_repository: Arc<DatabaseTableRepository>,
    pub database_view_repository: Arc<DatabaseViewRepository>,
    pub database_view_mapping_repository: Arc<DatabaseViewMappingRepository>,
    pub database_transformation_repository: Arc<DatabaseTransformationRepository>,
    pub sync_job_repository: Arc<SyncJobRepository>,
    pub metrics_summary_repository: Arc<MetricsSummaryRepository>,
    pub sync_manager: Arc<SyncManager>,
    pub metrics_use_case: Arc<MetricsUseCase>,
    pub database_model_repository: Arc<DatabaseModelRepository>,
    pub database_model_value_repository: Arc<DatabaseModelValueRepository>,
}

impl AppState {
    pub fn new(db: Database, jwt_secret: String, max_concurrent_jobs: usize) -> Self {
        let jwt_service = Arc::new(JwtService::new(jwt_secret));
        let company_repository = CompanyRepository::arc(db.clone());
        let user_repository = UserRepository::arc(db.clone());
        let database_configuration_repository = DatabaseConfigurationRepository::arc(db.clone());
        let database_column_repository = DatabaseColumnRepository::arc(db.clone());
        let database_table_repository = DatabaseTableRepository::arc(db.clone());
        let database_view_repository = DatabaseViewRepository::arc(db.clone());
        let database_view_mapping_repository = DatabaseViewMappingRepository::arc(db.clone());
        let database_transformation_repository = DatabaseTransformationRepository::arc(db.clone());
        let sync_job_repository = SyncJobRepository::arc(db.clone());
        let metrics_summary_repository = MetricsSummaryRepository::arc(db.clone());

        // Create SyncManager with configurable parallel workers from .env
        let sync_manager = Arc::new(SyncManager::new(
            max_concurrent_jobs,
            sync_job_repository.clone(),
            database_configuration_repository.clone(),
            database_view_repository.clone(),
            database_view_mapping_repository.clone(),
            database_transformation_repository.clone(),
            database_table_repository.clone(),
        ));
        
        // Get sync_status from SyncManager to create MetricsUseCase
        let sync_status = sync_manager.status.clone();
        
        // Create MetricsUseCase
        let metrics_use_case = Arc::new(MetricsUseCase::new(
            sync_job_repository.clone(),
            metrics_summary_repository.clone(),
            database_configuration_repository.clone(),
            database_view_repository.clone(),
            sync_status,
        ));
        let database_model_repository = DatabaseModelRepository::arc(db.clone());
        let database_model_value_repository = DatabaseModelValueRepository::arc(db.clone());

        Self {
            db,
            jwt_service,
            company_repository,
            user_repository,
            database_configuration_repository,
            database_column_repository,
            database_table_repository,
            database_view_repository,
            database_view_mapping_repository,
            database_transformation_repository,
            sync_job_repository,
            metrics_summary_repository,
            sync_manager,
            metrics_use_case,
            database_model_repository,
            database_model_value_repository,
        }
    }
    
    // Helper to get database reference for health checks
    pub fn database(&self) -> &Database {
        &self.db
    }
}
