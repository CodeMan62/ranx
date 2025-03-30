use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub window_seconds: u64,
}

#[derive(Debug)]
struct RequestWindow {
    timestamps: Vec<Instant>,
    total_requests: u64,
    blocked_requests: u64,
}

pub struct RateLimiter {
    config: RateLimitConfig,
    windows: Arc<RwLock<HashMap<String, RequestWindow>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        RateLimiter {
            config,
            windows: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, ip: &str) -> bool {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_seconds);
        let mut windows = self.windows.write().await;
        
        let window = windows.entry(ip.to_string()).or_insert(RequestWindow {
            timestamps: Vec::new(),
            total_requests: 0,
            blocked_requests: 0,
        });

        // Remove old timestamps
        window.timestamps.retain(|&ts| now.duration_since(ts) < window_duration);

        // Check if we're within limits
        if window.timestamps.len() as u32 >= self.config.requests_per_second {
            window.blocked_requests += 1;
            warn!("Rate limit exceeded for IP: {}", ip);
            false
        } else {
            window.timestamps.push(now);
            window.total_requests += 1;
            true
        }
    }

    pub async fn get_analytics(&self) -> HashMap<String, RateLimitAnalytics> {
        let windows = self.windows.read().await;
        let mut analytics = HashMap::new();

        for (ip, window) in windows.iter() {
            analytics.insert(ip.clone(), RateLimitAnalytics {
                total_requests: window.total_requests,
                blocked_requests: window.blocked_requests,
                current_window_requests: window.timestamps.len() as u64,
            });
        }

        analytics
    }
}

#[derive(Debug)]
pub struct RateLimitAnalytics {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub current_window_requests: u64,
} 
