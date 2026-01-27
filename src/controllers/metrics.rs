// Metrics Controller - Clean Architecture
// Responsabilidade: Apenas controle de requisições HTTP e WebSocket
// Toda lógica de negócio está no MetricsUseCase
use axum::{
    extract::{State, Query, WebSocketUpgrade, ws::{WebSocket, Message}},
    response::Response,
    Json,
};
use serde::Deserialize;
use tracing::{info, error, warn};
use futures::{sink::SinkExt, stream::StreamExt};

use crate::application::AppState;
use crate::utils::{ApiResponse, AppResult};

/// Query parameters para métricas
#[derive(Debug, Deserialize, Clone)]
pub struct MetricsQuery {
    /// Company ID (obrigatório)
    #[serde(rename = "companyId")]
    pub company_id: String,
    
    /// Intervalo de atualização em segundos para WebSocket (padrão: 3s)
    #[serde(rename = "updateInterval", default = "default_interval")]
    pub update_interval: u64,
}

fn default_interval() -> u64 {
    3
}

/// Mensagem WebSocket enviada ao cliente
#[derive(Debug, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum MetricsMessage {
    /// Snapshot inicial (ao conectar)
    Initial { data: crate::domain::entities::MetricsSummaryResponse },
    
    /// Update periódico (a cada N segundos)
    Update { data: crate::domain::entities::MetricsSummaryResponse },
    
    /// Erro
    Error { message: String },
}

/// GET /api/metrics/stream (WebSocket)
/// Endpoint WebSocket UNIDIRECIONAL - envia métricas em tempo real
pub async fn stream_metrics_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> Response {
    info!("📡 Nova conexão WebSocket de métricas para company_id: {}", query.company_id);
    
    let company_id = query.company_id.clone();
    let update_interval = query.update_interval;
    
    ws.on_upgrade(move |socket| handle_metrics_socket(socket, state, company_id, update_interval))
}

/// Handler do WebSocket (UNIDIRECIONAL - só envia)
async fn handle_metrics_socket(
    socket: WebSocket,
    state: AppState,
    company_id: String,
    update_interval: u64,
) {
    let (mut sender, mut receiver) = socket.split();
    
    info!("🔌 WebSocket conectado para company_id: {}", company_id);
    
    // PASSO 1: Enviar snapshot inicial imediatamente
    match state.metrics_use_case.get_or_calculate_metrics(&company_id).await {
        Ok(summary) => {
            let msg = MetricsMessage::Initial { data: summary };
            if let Ok(json) = serde_json::to_string(&msg) {
                if let Err(e) = sender.send(Message::Text(json)).await {
                    error!("❌ Erro ao enviar snapshot inicial: {}", e);
                    return;
                }
            }
        }
        Err(e) => {
            error!("❌ Erro ao calcular métricas iniciais: {}", e);
            let msg = MetricsMessage::Error {
                message: format!("Failed to calculate metrics: {}", e)
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = sender.send(Message::Text(json)).await;
            }
            return;
        }
    }
    
    // PASSO 2: Loop de atualização periódica
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(update_interval));
    
    loop {
        tokio::select! {
            // Timer: enviar update
            _ = interval.tick() => {
                match state.metrics_use_case.get_or_calculate_metrics(&company_id).await {
                    Ok(summary) => {
                        let msg = MetricsMessage::Update { data: summary };
                        match serde_json::to_string(&msg) {
                            Ok(json) => {
                                if let Err(e) = sender.send(Message::Text(json)).await {
                                    warn!("⚠️  Cliente desconectou: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("❌ Erro ao serializar métricas: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Erro ao calcular métricas: {}", e);
                    }
                }
            }
            
            // Receber mensagens do cliente (apenas para detectar desconexão)
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        info!("🔌 Cliente fechou conexão para company_id: {}", company_id);
                        break;
                    }
                    Some(Ok(Message::Ping(_))) => {
                        // Responder ping automaticamente (axum já faz isso)
                    }
                    None => {
                        info!("🔌 Conexão fechada para company_id: {}", company_id);
                        break;
                    }
                    Some(Err(e)) => {
                        error!("❌ Erro no WebSocket: {}", e);
                        break;
                    }
                    _ => {
                        // Ignorar outras mensagens (este é um WebSocket unidirecional)
                    }
                }
            }
        }
    }
    
    info!("🔚 WebSocket encerrado para company_id: {}", company_id);
}

/// GET /api/metrics (REST endpoint)
/// Retorna snapshot das métricas EM TEMPO REAL no momento da consulta
pub async fn get_metrics_rest(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> AppResult<Json<serde_json::Value>> {
    info!("📊 Request REST de métricas para company_id: {}", query.company_id);
    
    // Calcular métricas em tempo real
    let summary = state.metrics_use_case.get_or_calculate_metrics(&query.company_id).await?;
    
    let response = ApiResponse::success("Métricas calculadas com sucesso", summary);
    Ok(Json(serde_json::to_value(response).unwrap()))
}
