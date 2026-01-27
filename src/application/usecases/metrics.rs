// Metrics Use Case - Clean Architecture
// Responsabilidade: Calcular e persistir métricas em tempo real
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, error};

use crate::domain::entities::{MetricsSummary, MetricsSummaryResponse, CategoryStats};
use crate::infrastructure::repositories::{
    SyncJobRepository, 
    MetricsSummaryRepository,
    DatabaseConfigurationRepository,
    DatabaseViewRepository,
};
use crate::sync::status::SyncStatus;
use crate::utils::AppError;

/// Use Case para calcular métricas de uma empresa
#[derive(Clone)]
pub struct MetricsUseCase {
    sync_job_repo: Arc<SyncJobRepository>,
    metrics_summary_repo: Arc<MetricsSummaryRepository>,
    database_config_repo: Arc<DatabaseConfigurationRepository>,
    database_view_repo: Arc<DatabaseViewRepository>,
    sync_status: Arc<SyncStatus>,
}

impl MetricsUseCase {
    pub fn new(
        sync_job_repo: Arc<SyncJobRepository>,
        metrics_summary_repo: Arc<MetricsSummaryRepository>,
        database_config_repo: Arc<DatabaseConfigurationRepository>,
        database_view_repo: Arc<DatabaseViewRepository>,
        sync_status: Arc<SyncStatus>,
    ) -> Self {
        Self {
            sync_job_repo,
            metrics_summary_repo,
            database_config_repo,
            database_view_repo,
            sync_status,
        }
    }
    
    /// Calcula métricas em tempo real para uma empresa
    /// Retorna response com campos calculados + persistidos
    pub async fn calculate_metrics_summary(&self, company_id: &str) -> Result<MetricsSummaryResponse, AppError> {
        info!("📊 Calculando métricas para company_id: {}", company_id);
        
        // PASSO 1: Contar total de conexões (database configurations) - NÃO PERSISTE
        let total_connections = self.count_connections(company_id).await?;
        
        // PASSO 2: Contar total de integrações (database views) - NÃO PERSISTE
        let total_integrations = self.count_integrations(company_id).await?;
        
        // PASSO 3: Buscar todos os jobs (finalizados + em execução)
        let all_jobs = self.fetch_all_jobs(company_id).await?;
        
        // PASSO 4: Calcular taxas de sucesso e erro GERAIS
        let (success_rate, error_rate) = self.calculate_rates(&all_jobs);
        
        // PASSO 5: Calcular estatísticas por categoria (entity_type)
        let stats_by_category = self.calculate_stats_by_category(&all_jobs);
        
        info!("✅ Métricas calculadas: {} conexões, {} integrações, {:.2}% sucesso, {:.2}% erro, {} categorias",
            total_connections, total_integrations, success_rate, error_rate, stats_by_category.len());
        
        // PASSO 6: Criar/atualizar sumário (APENAS taxas são persistidas)
        let mut summary = MetricsSummary::new(company_id.to_string());
        summary.success_rate = success_rate;
        summary.error_rate = error_rate;
        
        // PASSO 7: Persistir no MongoDB (upsert - apenas taxas)
        let persisted_summary = self.metrics_summary_repo.upsert(&summary).await?;
        
        info!("💾 Sumário persistido no MongoDB");
        
        // PASSO 8: Criar response com campos calculados + persistidos
        let response = MetricsSummaryResponse::from_summary(
            persisted_summary,
            total_connections,
            total_integrations,
            stats_by_category,
        );
        
        Ok(response)
    }
    
    /// Busca métricas da empresa (sempre recalcula para garantir tempo real)
    pub async fn get_or_calculate_metrics(&self, company_id: &str) -> Result<MetricsSummaryResponse, AppError> {
        // Sempre recalcula para garantir dados em tempo real
        self.calculate_metrics_summary(company_id).await
    }
    
    /// Conta total de conexões (database configurations) de uma empresa
    async fn count_connections(&self, company_id: &str) -> Result<usize, AppError> {
        info!("🔍 Contando database_configurations para company_id: {}", company_id);
        
        let count = self.database_config_repo.count_by_company_id(company_id).await?;
        
        info!("✅ Total de database_configurations encontradas: {}", count);
        
        Ok(count as usize)
    }
    
    /// Conta total de integrações (database views) de uma empresa
    async fn count_integrations(&self, company_id: &str) -> Result<usize, AppError> {
        info!("🔍 Contando database_views para company_id: {}", company_id);
        
        let count = self.database_view_repo.count_by_company_id(company_id).await?;
        
        info!("✅ Total de database_views encontradas: {}", count);
        
        Ok(count as usize)
    }
    
    /// Busca todos os jobs (finalizados + em execução) de uma empresa
    /// 
    /// Estratégia:
    /// - Memória: APENAS jobs RUNNING (em tempo real)
    /// - MongoDB: Jobs finalizados (COMPLETED, FAILED, CANCELLED)
    async fn fetch_all_jobs(&self, company_id: &str) -> Result<Vec<crate::sync::job::SyncJob>, AppError> {
        let mut all_jobs = Vec::new();
        
        // PASSO 1: Buscar jobs RUNNING da memória (atualização em tempo real)
        let memory_jobs = self.sync_status.list_jobs_by_company(company_id).await;
        let running_count = memory_jobs.len();
        all_jobs.extend(memory_jobs);
        
        // PASSO 2: Buscar jobs FINALIZADOS do MongoDB (fonte única de verdade)
        let db_jobs = self.sync_job_repo.find_with_filters(
            Some(company_id.to_string()),
            None, // todos os status
            None, // sem filtro de data
            None,
            1,    // página 1
            10000, // limite alto para pegar todos
        ).await?;
        
        // Converter documentos MongoDB para SyncJob
        let finalized_count = db_jobs.len();
        for job_doc in db_jobs {
            all_jobs.push(job_doc.to_memory_job());
        }
        
        info!(
            "📈 Jobs encontrados: {} total ({} running na memória + {} finalizados no MongoDB)",
            all_jobs.len(),
            running_count,
            finalized_count
        );
        
        Ok(all_jobs)
    }
    
    /// Calcula taxas de sucesso e erro como MÉDIA das taxas de cada job
    /// 
    /// Exemplo:
    /// - Paciente: 80% sucesso, 20% erro
    /// - Atendimento: 100% sucesso, 0% erro
    /// Resultado: 90% sucesso, 10% erro (média simples)
    fn calculate_rates(&self, jobs: &[crate::sync::job::SyncJob]) -> (f64, f64) {
        if jobs.is_empty() {
            info!("⚠️  Nenhum job encontrado para calcular taxas");
            return (0.0, 0.0);
        }
        
        info!("🧮 Calculando taxas para {} jobs:", jobs.len());
        
        let mut total_success_rate = 0.0;
        let mut total_error_rate = 0.0;
        let mut valid_jobs = 0;
        
        for (index, job) in jobs.iter().enumerate() {
            let total_records = job.processed_records + job.failed_records;
            
            info!(
                "  📊 Job {}/{} ({}): processed={}, failed={}, total={}, status={:?}",
                index + 1,
                jobs.len(),
                &job.id[..8],
                job.processed_records,
                job.failed_records,
                total_records,
                job.status
            );
            
            if total_records > 0 {
                let job_success_rate = (job.processed_records as f64 / total_records as f64) * 100.0;
                let job_error_rate = (job.failed_records as f64 / total_records as f64) * 100.0;
                
                info!(
                    "    └─ Taxa deste job: {:.2}% sucesso, {:.2}% erro",
                    job_success_rate,
                    job_error_rate
                );
                
                total_success_rate += job_success_rate;
                total_error_rate += job_error_rate;
                valid_jobs += 1;
            } else {
                info!("    └─ Job sem records processados (ignorado no cálculo)");
            }
        }
        
        if valid_jobs > 0 {
            let avg_success = total_success_rate / valid_jobs as f64;
            let avg_error = total_error_rate / valid_jobs as f64;
            
            info!(
                "  ✅ Média final ({} jobs válidos): {:.2}% sucesso, {:.2}% erro",
                valid_jobs,
                avg_success,
                avg_error
            );
            
            (avg_success, avg_error)
        } else {
            info!("  ⚠️  Nenhum job válido para cálculo");
            (0.0, 0.0)
        }
    }
    
    /// Calcula estatísticas agrupadas por categoria (entity_type)
    /// 
    /// Agrupa todos os jobs pelo entity_type e soma:
    /// - total_records
    /// - processed_records
    /// - failed_records
    /// 
    /// E calcula as taxas de sucesso/erro para cada categoria
    fn calculate_stats_by_category(&self, jobs: &[crate::sync::job::SyncJob]) -> Vec<CategoryStats> {
        if jobs.is_empty() {
            info!("⚠️  Nenhum job para calcular stats por categoria");
            return vec![];
        }
        
        info!("📊 Agrupando {} jobs por entity_type", jobs.len());
        
        // Agrupar jobs por entity_type
        let mut category_map: HashMap<String, (u64, u64, u64)> = HashMap::new();
        
        for job in jobs {
            let entry = category_map.entry(job.entity_type.clone()).or_insert((0, 0, 0));
            
            // Total = APENAS o que foi processado (sucesso + falha)
            // NÃO usar job.total_records pois isso é o total no Oracle, não o que foi processado
            let total = job.processed_records + job.failed_records;
            
            entry.0 += total;                      // total_records
            entry.1 += job.processed_records;      // processed_records
            entry.2 += job.failed_records;         // failed_records
            
            info!(
                "  📦 {} - total:{} (processed:{} + failed:{})",
                job.entity_type,
                total,
                job.processed_records,
                job.failed_records
            );
        }
        
        // Converter HashMap para Vec<CategoryStats>
        let mut stats: Vec<CategoryStats> = category_map
            .into_iter()
            .map(|(entity_type, (total_records, processed_records, failed_records))| {
                // Calcular taxas
                let (success_rate, error_rate) = if total_records > 0 {
                    let success = (processed_records as f64 / total_records as f64) * 100.0;
                    let error = (failed_records as f64 / total_records as f64) * 100.0;
                    (success, error)
                } else {
                    (0.0, 0.0)
                };
                
                info!(
                    "  ✅ {}: total={}, processed={}, failed={}, success={:.2}%, error={:.2}%",
                    entity_type,
                    total_records,
                    processed_records,
                    failed_records,
                    success_rate,
                    error_rate
                );
                
                CategoryStats {
                    entity_type,
                    total_records,
                    processed_records,
                    failed_records,
                    success_rate,
                    error_rate,
                }
            })
            .collect();
        
        // Ordenar por entity_type para resposta consistente
        stats.sort_by(|a, b| a.entity_type.cmp(&b.entity_type));
        
        info!("✅ {} categorias calculadas", stats.len());
        
        stats
    }
}
