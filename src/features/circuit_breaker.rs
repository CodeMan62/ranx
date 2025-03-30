use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Not allowing requests
    HalfOpen,  // Testing if service is back
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub reset_timeout: Duration,
    pub half_open_timeout: Duration,
}

struct CircuitMetrics {
    failures: u32,
    last_failure: Instant,
    state: CircuitState,
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
}

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    metrics: Arc<RwLock<HashMap<String, CircuitMetrics>>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        CircuitBreaker {
            config,
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn pre_request(&self, backend: &str) -> bool {
        let mut metrics = self.metrics.write().await;
        let metric = metrics.entry(backend.to_string()).or_insert(CircuitMetrics {
            failures: 0,
            last_failure: Instant::now(),
            state: CircuitState::Closed,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
        });

        metric.total_requests += 1;

        match metric.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if Instant::now().duration_since(metric.last_failure) > self.config.reset_timeout {
                    info!("Circuit moving to half-open state for backend: {}", backend);
                    metric.state = CircuitState::HalfOpen;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub async fn record_success(&self, backend: &str) {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(backend) {
            metric.successful_requests += 1;
            if metric.state == CircuitState::HalfOpen {
                info!("Circuit closed for backend: {}", backend);
                metric.state = CircuitState::Closed;
                metric.failures = 0;
            }
        }
    }

    pub async fn record_failure(&self, backend: &str) {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(backend) {
            metric.failed_requests += 1;
            metric.failures += 1;
            metric.last_failure = Instant::now();

            if metric.failures >= self.config.failure_threshold {
                warn!("Circuit opened for backend: {}", backend);
                metric.state = CircuitState::Open;
            }
        }
    }

    pub async fn get_metrics(&self) -> HashMap<String, CircuitBreakerMetrics> {
        let metrics = self.metrics.read().await;
        let mut result = HashMap::new();

        for (backend, metric) in metrics.iter() {
            result.insert(backend.clone(), CircuitBreakerMetrics {
                state: metric.state.clone(),
                total_requests: metric.total_requests,
                successful_requests: metric.successful_requests,
                failed_requests: metric.failed_requests,
                current_failures: metric.failures,
            });
        }

        result
    }
}

#[derive(Debug)]
pub struct CircuitBreakerMetrics {
    pub state: CircuitState,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub current_failures: u32,
} 
