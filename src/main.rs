mod settings;
mod domain;
mod application;
mod infrastructure;
mod controllers;
mod utils;
mod core;
mod seed;
mod sync;

use axum::{
    http::Method,
};
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "interhealth_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = settings::Config::from_env()?;
    let db = infrastructure::adapters::mongodb::connect(&config.mongo_url).await?;

    tracing::info!("[Database] MongoDB connection initialized");

    // Check if we should run seed
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "seed" {
        let app_state = application::AppState::new(db, config.jwt_secret, config.max_concurrent_jobs);
        seed::seed_database(
            app_state.company_repository,
            app_state.user_repository,
            app_state.database_configuration_repository,
            app_state.database_view_repository,
            app_state.database_table_repository,
            app_state.database_column_repository,
            app_state.database_view_mapping_repository,
            app_state.database_model_repository,
            app_state.database_transformation_repository,
            app_state.database_model_value_repository,
        ).await?;
        return Ok(());
    }

    let mut app_state = application::AppState::new(db, config.jwt_secret, config.max_concurrent_jobs);

    // Initialize SyncManager (start background workers)
    tracing::info!("🚀 Initializing SyncManager...");
    {
        // Get mutable reference to sync_manager
        let sync_manager = Arc::get_mut(&mut app_state.sync_manager)
            .expect("Failed to get mutable reference to SyncManager");
        sync_manager.start().await;
    }
    tracing::info!("✅ SyncManager initialized with background workers running!");

    // CORS configuration - specify explicit origins when using credentials
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ]);
        // .allow_credentials(true);

    let app = controllers::create_routes(app_state)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.app_port));
    tracing::info!("InterHealth API - Nossa revolução começa aqui");
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
