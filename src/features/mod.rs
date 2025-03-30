pub mod circuit_breaker;
pub mod metrics;
pub mod ratelimit;

use std::sync::Arc;
use std::time::Duration;

pub struct Features {
    pub rate_limiter: Arc<ratelimit::RateLimiter>,
    pub circuit_breaker: Arc<circuit_breaker::CircuitBreaker>,
    pub metrics_collector: Arc<metrics::MetricsCollector>,
}

impl Features {
    pub fn new() -> Self {
        let rate_limiter = Arc::new(ratelimit::RateLimiter::new(
            ratelimit::RateLimitConfig {
                requests_per_second: 100,
                burst_size: 50,
                window_seconds: 60,
            }
        ));

        let circuit_breaker = Arc::new(circuit_breaker::CircuitBreaker::new(
            circuit_breaker::CircuitBreakerConfig {
                failure_threshold: 5,
                reset_timeout: Duration::from_secs(30),
                half_open_timeout: Duration::from_secs(5),
            }
        ));

        let metrics_collector = Arc::new(metrics::MetricsCollector::new());

        Features {
            rate_limiter,
            circuit_breaker,
            metrics_collector,
        }
    }
} 
