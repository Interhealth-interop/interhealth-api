use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use crate::utils::utils::{date_format, object_id_format};

/// Sumário de métricas agregadas para uma empresa
/// Persiste no MongoDB e é atualizado em tempo real
/// 
/// NOTA: total_connections e total_integrations NÃO são persistidos
/// Eles vêm de count_documents direto nas collections
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsSummary {
    #[serde(
        rename(serialize = "id", deserialize = "_id"),
        skip_serializing_if = "Option::is_none",
        with = "object_id_format"
    )]
    pub id: Option<ObjectId>,
    
    /// Company ID (chave única por empresa)
    pub company_id: String,
    
    /// Taxa de sucesso (média das taxas de todas as execuções)
    pub success_rate: f64,
    
    /// Taxa de erro (média das taxas de todas as execuções)
    pub error_rate: f64,
    
    /// Timestamp da última atualização
    #[serde(with = "date_format")]
    pub updated_at: DateTime<Utc>,
    
    /// Timestamp de criação
    #[serde(with = "date_format")]
    pub created_at: DateTime<Utc>,
}

impl MetricsSummary {
    /// Cria um novo sumário de métricas
    pub fn new(company_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            company_id,
            success_rate: 0.0,
            error_rate: 0.0,
            updated_at: now,
            created_at: now,
        }
    }
    
    /// Atualiza o timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Estatísticas por categoria (entity_type)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryStats {
    /// Tipo de entidade (PATIENT, ENCOUNTER, etc)
    pub entity_type: String,
    
    /// Total de registros processados (soma de todos os jobs)
    pub total_records: u64,
    
    /// Registros processados com sucesso (soma)
    pub processed_records: u64,
    
    /// Registros que falharam (soma)
    pub failed_records: u64,
    
    /// Taxa de sucesso (%)
    pub success_rate: f64,
    
    /// Taxa de erro (%)
    pub error_rate: f64,
}

/// Resposta da API com métricas calculadas
/// Inclui campos calculados em tempo real que NÃO são persistidos
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsSummaryResponse {
    #[serde(
        rename(serialize = "id", deserialize = "_id"),
        skip_serializing_if = "Option::is_none",
        with = "object_id_format"
    )]
    pub id: Option<ObjectId>,
    
    pub company_id: String,
    
    /// Total de conexões (calculado via count_documents)
    pub total_connections: usize,
    
    /// Total de integrações (calculado via count_documents)
    pub total_integrations: usize,
    
    pub success_rate: f64,
    pub error_rate: f64,
    
    /// Estatísticas agrupadas por categoria (entity_type)
    pub stats_by_category: Vec<CategoryStats>,
    
    #[serde(with = "date_format")]
    pub updated_at: DateTime<Utc>,
    
    #[serde(with = "date_format")]
    pub created_at: DateTime<Utc>,
}

impl MetricsSummaryResponse {
    /// Cria resposta a partir de MetricsSummary + counts calculados
    pub fn from_summary(
        summary: MetricsSummary,
        total_connections: usize,
        total_integrations: usize,
        stats_by_category: Vec<CategoryStats>,
    ) -> Self {
        Self {
            id: summary.id,
            company_id: summary.company_id,
            total_connections,
            total_integrations,
            success_rate: summary.success_rate,
            error_rate: summary.error_rate,
            stats_by_category,
            updated_at: summary.updated_at,
            created_at: summary.created_at,
        }
    }
}
