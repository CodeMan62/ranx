use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct RequestMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_response_time: Duration,
    pub min_response_time: Option<Duration>,
    pub max_response_time: Option<Duration>,
    pub status_codes: HashMap<u16, u64>,
}

pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, RequestMetrics>>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        MetricsCollector {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    pub async fn record_request(&self, backend: &str, start: Instant, status: u16, is_error: bool) {
        let duration = start.elapsed();
        let mut metrics = self.metrics.write().await;
        
        let backend_metrics = metrics.entry(backend.to_string()).or_default();
        
        // Update request counts
        backend_metrics.total_requests += 1;
        if is_error {
            backend_metrics.failed_requests += 1;
        } else {
            backend_metrics.successful_requests += 1;
        }

        // Update response time metrics
        backend_metrics.total_response_time += duration;
        match backend_metrics.min_response_time {
            None => backend_metrics.min_response_time = Some(duration),
            Some(min) if duration < min => backend_metrics.min_response_time = Some(duration),
            _ => {}
        }
        match backend_metrics.max_response_time {
            None => backend_metrics.max_response_time = Some(duration),
            Some(max) if duration > max => backend_metrics.max_response_time = Some(duration),
            _ => {}
        }

        // Update status code counts
        *backend_metrics.status_codes.entry(status).or_default() += 1;
    }

    pub async fn get_metrics(&self) -> HashMap<String, MetricsSummary> {
        let metrics = self.metrics.read().await;
        let mut summaries = HashMap::new();

        for (backend, metric) in metrics.iter() {
            let avg_response_time = if metric.total_requests > 0 {
                metric.total_response_time.as_secs_f64() / metric.total_requests as f64
            } else {
                0.0
            };

            summaries.insert(backend.clone(), MetricsSummary {
                uptime: self.start_time.elapsed(),
                total_requests: metric.total_requests,
                successful_requests: metric.successful_requests,
                failed_requests: metric.failed_requests,
                success_rate: if metric.total_requests > 0 {
                    (metric.successful_requests as f64 / metric.total_requests as f64) * 100.0
                } else {
                    0.0
                },
                avg_response_time,
                min_response_time: metric.min_response_time,
                max_response_time: metric.max_response_time,
                status_code_distribution: metric.status_codes.clone(),
            });
        }

        summaries
    }

    pub async fn log_metrics(&self) {
        let metrics = self.get_metrics().await;
        for (backend, summary) in metrics {
            info!(
                "Backend {} metrics:\n\
                 - Uptime: {:.2} seconds\n\
                 - Total requests: {}\n\
                 - Success rate: {:.2}%\n\
                 - Avg response time: {:.2}ms",
                backend,
                summary.uptime.as_secs_f64(),
                summary.total_requests,
                summary.success_rate,
                summary.avg_response_time * 1000.0
            );
        }
    }
}

#[derive(Debug)]
pub struct MetricsSummary {
    pub uptime: Duration,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub avg_response_time: f64,
    pub min_response_time: Option<Duration>,
    pub max_response_time: Option<Duration>,
    pub status_code_distribution: HashMap<u16, u64>,
} 
