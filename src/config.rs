use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    
    /// Backends configuration
    pub backends: HashMap<String, BackendConfig>,
    
    /// Routes configuration
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    /// Address to bind the proxy server to
    pub listen_addr: SocketAddr,
    
    /// TLS configuration (optional)
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TlsConfig {
    /// Path to the certificate file
    pub cert_path: String,
    
    /// Path to the private key file
    pub key_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BackendConfig {
    /// List of backend server addresses
    pub servers: Vec<String>,
    
    /// Load balancing method (round-robin, least-connections, etc.)
    #[serde(default = "default_load_balancing")]
    pub load_balancing: String,
    
    /// Health check configuration (optional)
    pub health_check: Option<HealthCheckConfig>,
    
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HealthCheckConfig {
    /// Path to use for health check
    pub path: String,
    
    /// Interval between health checks in seconds
    #[serde(default = "default_health_interval")]
    pub interval: u64,
    
    /// Timeout for health check in seconds
    #[serde(default = "default_health_timeout")]
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RouteConfig {
    /// Path pattern to match
    pub path: String,
    
    /// Backend to route to
    pub backend: String,
    
    /// Strip prefix from request path
    #[serde(default)]
    pub strip_prefix: bool,
}

fn default_load_balancing() -> String {
    "round-robin".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_health_interval() -> u64 {
    10
}

fn default_health_timeout() -> u64 {
    5
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let config_content = fs::read_to_string(path)
        .context("Failed to read configuration file")?;
    
    let config: Config = serde_yaml::from_str(&config_content)
        .context("Failed to parse configuration file")?;
    
    validate_config(&config)?;
    
    Ok(config)
}

fn validate_config(config: &Config) -> Result<()> {
    // Ensure all backends referenced in routes exist
    for route in &config.routes {
        if !config.backends.contains_key(&route.backend) {
            anyhow::bail!("Backend '{}' referenced in route not found", route.backend);
        }
    }
    
    // Ensure backends have at least one server
    for (name, backend) in &config.backends {
        if backend.servers.is_empty() {
            anyhow::bail!("Backend '{}' has no servers", name);
        }
    }
    
    Ok(())
} 
