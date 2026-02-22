use axum::{
    Router,
    routing::{get, post, put, delete},
};

use crate::application::AppState;
use crate::controllers::{
    user, company, auth, health, database_configuration, database_column,
    database_table, database_view, database_view_mapping,
    target_integration, integration_control,
    sync, metrics, database_model
};

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        // Health route
        .route("/health", get(health::check_health))
        // Auth routes
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        .route("/auth/refresh", post(auth::refresh_token))
        // User routes
        .route("/user", post(user::create_user))
        .route("/user", get(user::get_all_users))
        .route("/user/:id", get(user::get_user_by_id))
        .route("/user/:id", put(user::update_user))
        .route("/user/:id", delete(user::delete_user))
        // Company routes
        .route("/company", post(company::create_company))
        .route("/company", get(company::get_all_companies))
        .route("/company/:id", get(company::get_company_by_id))
        .route("/company/code/:code", get(company::get_company_by_code))
        .route("/company/cnpj/:cnpj", get(company::get_company_by_cnpj))
        .route("/company/:id", put(company::update_company))
        .route("/company/:id/status", put(company::change_company_status))
        .route("/company/:id", delete(company::delete_company))
        
        // Database Configuration routes
        .route("/database-configuration", post(database_configuration::create_database_configuration))
        .route("/database-configuration", get(database_configuration::get_all_database_configurations))
        .route("/database-configuration/:id", get(database_configuration::get_database_configuration_by_id))
        .route("/database-configuration/test-connection", post(database_configuration::test_connection))
        .route("/database-configuration/:id", put(database_configuration::update_database_configuration))
        .route("/database-configuration/:id", delete(database_configuration::delete_database_configuration))
        
        // Database Column routes
        .route("/database-columns", post(database_column::create_database_column))
        .route("/database-columns", get(database_column::get_all_database_columns))
        .route("/database-columns/:id", get(database_column::get_database_column_by_id))
        .route("/database-columns/table/:table_id", get(database_column::get_database_columns_by_table_id))
        .route("/database-columns/:id", put(database_column::update_database_column))
        .route("/database-columns/:id", delete(database_column::delete_database_column))
        
        // Database Table routes
        .route("/database-tables", post(database_table::create_database_table))
        .route("/database-tables", get(database_table::get_all_database_tables))
        .route("/database-tables/connection/:connection_id", get(database_table::get_database_tables_by_connection_id))
        .route("/database-tables/:id", get(database_table::get_database_table_by_id))
        .route("/database-tables/:id", put(database_table::update_database_table))
        .route("/database-tables/:id", delete(database_table::delete_database_table))
        
        // Database View routes
        .route("/database-view", post(database_view::create_database_view))
        .route("/database-view", get(database_view::get_all_database_views))
        .route("/database-view/:id", get(database_view::get_database_view_by_id))
        .route("/database-view/:id", put(database_view::update_database_view))
        .route("/database-view/:id", delete(database_view::delete_database_view))

        // Target Integration routes
        .route("/target-integration", post(target_integration::create_target_integration))
        .route("/target-integration/:id", get(target_integration::get_target_integration_by_id))
        .route("/target-integration/view/:view_id", get(target_integration::get_target_integration_by_database_view_id))
        .route("/target-integration/:id", put(target_integration::update_target_integration))
        .route("/target-integration/:id", delete(target_integration::delete_target_integration))

        // Integration Control routes
        .route("/integration-control", post(integration_control::create_integration_control))
        .route("/integration-control/:id", get(integration_control::get_integration_control_by_id))
        .route("/integration-control/view/:view_id", get(integration_control::get_integration_controls_by_database_view_id))
        .route("/integration-control/:id", put(integration_control::update_integration_control))
        .route("/integration-control/:id", delete(integration_control::delete_integration_control))
        
        // Database View Mapping routes
        .route("/database-view-mapping", post(database_view_mapping::create_database_view_mapping))
        .route("/database-view-mapping", get(database_view_mapping::get_all_database_view_mappings))
        .route("/database-view-mapping/:id", get(database_view_mapping::get_database_view_mapping_by_id))
        .route("/database-view-mapping/view/:view_id", get(database_view_mapping::get_database_view_mappings_by_view_id))
        .route("/database-view-mapping/view/:view_id/preview", get(database_view_mapping::get_database_view_mappings_preview))
        .route("/database-view-mapping/:id", put(database_view_mapping::update_database_view_mapping))
        .route("/database-view-mapping/:id", delete(database_view_mapping::delete_database_view_mapping))
        
        // Sync routes (Background synchronization)
        .route("/sync/init", post(sync::start_sync))  // Inicia novo job ou retoma pausado
        .route("/sync/jobs/:job_id", get(sync::get_sync_status))  // Status de job específico
        .route("/sync/jobs/:job_id/pause", post(sync::pause_job))  // Pausar job
        .route("/sync/jobs/:job_id/resume", post(sync::resume_job))  // Retomar job pausado
        .route("/sync/jobs/:job_id/restart", post(sync::restart_job))  // Reexecutar job (qualquer status)
        .route("/sync/stats", get(sync::get_sync_stats))  // Estatísticas gerais
        .route("/sync/stats/memory", get(sync::get_memory_jobs))  // Jobs em memória (paginado)
        .route("/sync/stats/persisted", get(sync::get_persisted_jobs))  // Jobs no MongoDB (paginado)
        
        // Metrics routes (Real-time dashboard metrics)
        .route("/metrics/stream", get(metrics::stream_metrics_ws))  // WebSocket (tempo real)
        .route("/metrics", get(metrics::get_metrics_rest))           // REST (snapshot único)
        
        // Database Model routes
        .route("/database-model", post(database_model::create_database_model))
        .route("/database-model", get(database_model::get_all_database_models))
        .route("/database-model/:id", get(database_model::get_database_model_by_id))
        .route("/database-model/:id/model-values", get(database_model::get_database_model_mapping_values))
        .route("/database-model/:id/model-values", post(database_model::upsert_database_model_mapping_value))
        .route("/database-model/:id/model-values/:value_id", put(database_model::update_database_model_value_mapping))
        .route("/database-model/:id/model-values/:value_id", delete(database_model::delete_database_model_value))
        .route("/database-model/:id/model-values/:value_id/connection/:connection_id", put(database_model::update_database_model_value_connection_mapping))
        .route("/database-model/:id/model-values/:value_id/connection/:connection_id", delete(database_model::delete_database_model_value_connection_mapping))
        .route("/database-model/:id", put(database_model::update_database_model))
        .route("/database-model/:id", delete(database_model::delete_database_model))
        
        .with_state(state)
}
