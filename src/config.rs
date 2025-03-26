use std::{collections::HashMap, fs, net::SocketAddr, path::Path};

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config{
    pub server: ServerConfig,

    pub backends: HashMap<String, BackendConfig>,

    pub routes: Vec<RouteConfig>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig{
    pub listen_addr: SocketAddr,

    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig{
    // path to the certificate file
    pub cert_path: String,
    // path to the private key  file
    pub key_path: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig{
    pub servers: Vec<String>,

    #[serde(default="default_load_balancing")]
    pub load_balancing: String,

    pub health_check: Option<HealthCheckConfig>,
    #[serde(default="default_timeout")]
    pub timeout: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig{
    pub path: String,

    #[serde(default="default_health_interval")]
    pub interval: u64,

    #[serde(default="default_health_timeout")]
    pub timeout: u64,
}

fn default_timeout() -> u64{
    30
}
fn default_health_timeout() -> u64{
    5
}
fn default_health_interval() -> u64{
    10
}
fn default_load_balancing() -> String{
    "round-robin".to_string()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig{
    pub path: String,

    pub backend: String,

    #[serde(default)]
    pub strip_prefix: bool
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config>{
    let config_content = fs::read_to_string(path)
        .context("Failed to read config file")?;

    let config: Config = serde_yaml::from_str(&config_content)
        .context("Failed to parse config file")?;
    validate_config(&config)?;
    Ok(config)
}


fn validate_config(config: &Config) -> Result<()> {
    // Ensure all backend referenced in routes exist
    for route in &config.routes {
        if !config.backends.contains_key(&route.backend) {
            anyhow::bail!("Backend '{}' referenced in route not found", route.backend);
        }
    }

    // Ensure backends have at least one server
    for (name, backend) in &config.backends {
        if backend.servers.is_empty() {
            anyhow::bail!("Backend '{}' has no servers ", name);
        }
    }
    Ok(())
}

