use axum::{extract::State, Json};
use serde::Serialize;
use crate::application::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub message: String,
}

pub async fn check_health(State(state): State<AppState>) -> Json<HealthResponse> {
    // Try to ping the database
    let db = state.database();
    let db_status = match db.run_command(mongodb::bson::doc! { "ping": 1 }, None).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status.to_string(),
        message: "InterHealth API is running".to_string(),
    })
}
