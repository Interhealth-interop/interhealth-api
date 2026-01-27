use dotenvy::dotenv;
use std::env;

use crate::utils::AppError;

#[derive(Debug, Clone)]
pub struct Config {
    pub mongo_url: String,
    pub app_port: u16,
    pub jwt_secret: String,
    pub max_concurrent_jobs: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenv().ok();

        let mongo_url = env::var("MONGO_URL")
            .map_err(|_| AppError::ConfigError("MONGO_URL must be set".to_string()))?;

        let app_port = env::var("APP_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|_| AppError::ConfigError("Invalid APP_PORT".to_string()))?;

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

        let max_concurrent_jobs = env::var("MAX_CONCURRENT_JOBS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|_| AppError::ConfigError("Invalid MAX_CONCURRENT_JOBS".to_string()))?;

        Ok(Config {
            mongo_url,
            app_port,
            jwt_secret,
            max_concurrent_jobs,
        })
    }
}
