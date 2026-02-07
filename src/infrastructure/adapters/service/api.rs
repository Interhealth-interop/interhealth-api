use crate::utils::AppError;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use serde_json::Value;
use std::time::Duration;

/// API connection configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub auth_type: Option<String>,
    pub credentials: Option<String>,
}

impl ApiConfig {
    /// Create API config from parameters
    pub fn new(host: String, auth_type: Option<String>, credentials: Option<String>) -> Self {
        Self {
            host,
            auth_type,
            credentials,
        }
    }
}

/// API connector for HTTP/REST integrations
pub struct ApiConnector {
    config: ApiConfig,
    client: Client,
}

impl ApiConnector {
    /// Create a new API connector
    pub async fn new(host: &str, auth_type: Option<String>, credentials: Option<String>) -> Result<Self, AppError> {
        let config = ApiConfig::new(host.to_string(), auth_type, credentials);
        
        // Build HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::DatabaseError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            config,
            client,
        })
    }

    /// Create connector from config
    pub async fn from_config(config: ApiConfig) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::DatabaseError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            config,
            client,
        })
    }

    /// Build headers with authentication
    fn build_headers(&self) -> Result<HeaderMap, AppError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let (Some(auth_type), Some(credentials)) = (&self.config.auth_type, &self.config.credentials) {
            match auth_type.to_lowercase().as_str() {
                "bearer" => {
                    let auth_value = format!("Bearer {}", credentials);
                    headers.insert(
                        AUTHORIZATION,
                        HeaderValue::from_str(&auth_value)
                            .map_err(|e| AppError::BadRequest(format!("Invalid authorization header: {}", e)))?
                    );
                }
                "basic" => {
                    let auth_value = format!("Basic {}", credentials);
                    headers.insert(
                        AUTHORIZATION,
                        HeaderValue::from_str(&auth_value)
                            .map_err(|e| AppError::BadRequest(format!("Invalid authorization header: {}", e)))?
                    );
                }
                "apikey" | "api_key" => {
                    headers.insert(
                        "X-API-Key",
                        HeaderValue::from_str(credentials)
                            .map_err(|e| AppError::BadRequest(format!("Invalid API key: {}", e)))?
                    );
                }
                _ => {
                    // Custom auth type - add as Authorization header
                    headers.insert(
                        AUTHORIZATION,
                        HeaderValue::from_str(credentials)
                            .map_err(|e| AppError::BadRequest(format!("Invalid authorization header: {}", e)))?
                    );
                }
            }
        }

        Ok(headers)
    }

    /// Test the API connection
    pub async fn test_connection(&self) -> Result<bool, AppError> {
        let headers = self.build_headers()?;
        
        // Try to make a simple GET request to the host
        let response = self.client
            .get(&self.config.host)
            .headers(headers)
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(format!("API connection failed: {}", e)))?;

        // Consider 2xx, 3xx, 401, and 403 as "connected" (server is reachable)
        // 401/403 means auth failed but connection works
        let status = response.status();
        Ok(status.is_success() || status.is_redirection() || status.as_u16() == 401 || status.as_u16() == 403)
    }

    /// Execute a GET request
    pub async fn get(&self, path: &str) -> Result<Value, AppError> {
        let headers = self.build_headers()?;
        let url = format!("{}{}", self.config.host.trim_end_matches('/'), path);
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(format!("GET request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::DatabaseError(format!(
                "API request failed with status: {}",
                response.status()
            )));
        }

        let json = response.json::<Value>()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse JSON response: {}", e)))?;

        Ok(json)
    }

    /// Execute a POST request
    pub async fn post(&self, path: &str, body: Value) -> Result<Value, AppError> {
        let headers = self.build_headers()?;
        let url = format!("{}{}", self.config.host.trim_end_matches('/'), path);
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(format!("POST request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::DatabaseError(format!(
                "API request failed with status: {}",
                response.status()
            )));
        }

        let json = response.json::<Value>()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse JSON response: {}", e)))?;

        Ok(json)
    }

    /// Execute a PUT request
    pub async fn put(&self, path: &str, body: Value) -> Result<Value, AppError> {
        let headers = self.build_headers()?;
        let url = format!("{}{}", self.config.host.trim_end_matches('/'), path);
        
        let response = self.client
            .put(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(format!("PUT request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::DatabaseError(format!(
                "API request failed with status: {}",
                response.status()
            )));
        }

        let json = response.json::<Value>()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse JSON response: {}", e)))?;

        Ok(json)
    }

    /// Execute a DELETE request
    pub async fn delete(&self, path: &str) -> Result<Value, AppError> {
        let headers = self.build_headers()?;
        let url = format!("{}{}", self.config.host.trim_end_matches('/'), path);
        
        let response = self.client
            .delete(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| AppError::DatabaseError(format!("DELETE request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::DatabaseError(format!(
                "API request failed with status: {}",
                response.status()
            )));
        }

        let json = response.json::<Value>()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse JSON response: {}", e)))?;

        Ok(json)
    }

    /// Get connection status
    pub fn is_connected(&self) -> bool {
        true // HTTP client is always "connected" until test_connection is called
    }

    /// Get connection configuration
    pub fn get_config(&self) -> &ApiConfig {
        &self.config
    }
}
