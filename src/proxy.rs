use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::future::BoxFuture;
use hyper::client::{Client, HttpConnector};
use hyper::{Body, Request, Response, StatusCode, Uri};
use hyper::header::{HeaderMap, HeaderName, HeaderValue};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::{BackendConfig, Config, RouteConfig};
use crate::error::{ProxyError, ProxyResult};
use crate::features::Features;

// HTTP client with connection pooling
static HTTP_CLIENT: Lazy<Client<HttpConnector>> = Lazy::new(|| {
    let mut connector = HttpConnector::new();
    connector.set_nodelay(true);
    connector.set_keepalive(Some(Duration::from_secs(30)));
    
    Client::builder()
        .pool_idle_timeout(Duration::from_secs(30))
        .build(connector)
});

pub struct ProxyService {
    config: Config,
    routes: Vec<RouteConfig>,
    backends: HashMap<String, BackendState>,
    features: Features,
}

struct BackendState {
    config: BackendConfig,
    next_server_index: RwLock<usize>,
}

pub fn create_proxy_service(config: Config) -> ProxyService {
    let backends = config.backends
        .iter()
        .map(|(name, backend_config)| {
            let backend_state = BackendState {
                config: backend_config.clone(),
                next_server_index: RwLock::new(0),
            };
            
            (name.clone(), backend_state)
        })
        .collect();
    
    ProxyService {
        routes: config.routes.clone(),
        backends,
        config,
        features: Features::new(),
    }
}

impl ProxyService {
    pub async fn proxy_request(&self, req: Request<Body>) -> ProxyResult<Response<Body>> {
        let start_time = Instant::now();
        let path = req.uri().path();
        let client_ip = get_client_ip(&req).unwrap_or("unknown".to_string());
        
        debug!("Received request for path: {} from {}", path, client_ip);
        
        // Check rate limit
        if !self.features.rate_limiter.check_rate_limit(&client_ip).await {
            return Err(ProxyError::RateLimitExceeded);
        }
        
        // Find matching route
        let route = self.find_route(path)
            .ok_or_else(|| ProxyError::RouteNotFound(path.to_string()))?;
        
        // Get backend for the route
        let backend = self.backends.get(&route.backend)
            .ok_or_else(|| ProxyError::BackendError(format!("Backend not found: {}", route.backend)))?;
        
        // Check circuit breaker
        if !self.features.circuit_breaker.pre_request(&route.backend).await {
            return Err(ProxyError::CircuitBreakerOpen);
        }
        
        // Select a backend server using load balancing
        let target_server = self.select_backend_server(backend).await?;
        
        // Build the target URI
        let target_uri = self.build_target_uri(&req, route, target_server).await?;
        
        debug!("Forwarding request to: {}", target_uri);
        
        // Forward the request to the target server
        match self.forward_request(req, target_uri).await {
            Ok(response) => {
                // Record success metrics
                self.features.circuit_breaker.record_success(&route.backend).await;
                self.features.metrics_collector.record_request(
                    &route.backend,
                    start_time,
                    response.status().as_u16(),
                    false
                ).await;
                
                Ok(response)
            }
            Err(e) => {
                // Record failure metrics
                self.features.circuit_breaker.record_failure(&route.backend).await;
                self.features.metrics_collector.record_request(
                    &route.backend,
                    start_time,
                    500,
                    true
                ).await;
                
                Err(e)
            }
        }
    }
    
    fn find_route(&self, path: &str) -> Option<&RouteConfig> {
        self.routes.iter()
            .find(|route| path.starts_with(&route.path))
    }
    
    async fn select_backend_server(&self, backend: &BackendState) -> ProxyResult<String> {
        let servers = &backend.config.servers;
        if servers.is_empty() {
            return Err(ProxyError::NoHealthyBackends);
        }
        
        let mut index = backend.next_server_index.write().await;
        let server = servers[*index].clone();
        
        // Update the index for the next request
        *index = (*index + 1) % servers.len();
        
        Ok(server)
    }
    
    async fn build_target_uri(&self, req: &Request<Body>, route: &RouteConfig, server: String) -> ProxyResult<Uri> {
        let path = req.uri().path();
        let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
        
        let target_path = if route.strip_prefix {
            path.strip_prefix(&route.path)
                .unwrap_or(path)
                .to_string()
        } else {
            path.to_string()
        };
        
        let target_path = if target_path.starts_with('/') || target_path.is_empty() {
            target_path
        } else {
            format!("/{}", target_path)
        };
        
        let uri = format!("{}{}{}", server, target_path, query);
        
        Uri::try_from(uri)
            .map_err(|e| ProxyError::BackendError(format!("Invalid URI: {}", e)))
    }
    
    async fn forward_request(&self, mut req: Request<Body>, target_uri: Uri) -> ProxyResult<Response<Body>> {
        let (parts, body) = req.into_parts();
        
        let mut outgoing_req = Request::builder()
            .method(parts.method)
            .uri(target_uri)
            .version(parts.version);
        
        let headers = outgoing_req.headers_mut().unwrap();
        copy_headers(&parts.headers, headers);
        
        if let Some(host) = parts.uri.host() {
            if let Ok(value) = HeaderValue::from_str(host) {
                headers.insert("X-Forwarded-Host", value);
            }
        }
        
        if let Some(port) = parts.uri.port_u16() {
            if let Ok(value) = HeaderValue::from_str(&port.to_string()) {
                headers.insert("X-Forwarded-Port", value);
            }
        }
        
        if let Some(scheme) = parts.uri.scheme_str() {
            if let Ok(value) = HeaderValue::from_str(scheme) {
                headers.insert("X-Forwarded-Proto", value);
            }
        }
        
        let outgoing_req = outgoing_req.body(body)
            .map_err(|e| ProxyError::BackendError(format!("Failed to build request: {}", e)))?;
        
        let timeout_duration = Duration::from_secs(backend_timeout(&self.backends, self.find_route(outgoing_req.uri().path())));
        
        let response = tokio::time::timeout(
            timeout_duration, 
            HTTP_CLIENT.request(outgoing_req)
        ).await
            .map_err(|_| ProxyError::TimeoutError(format!("Request timed out after {} seconds", timeout_duration.as_secs())))?
            .map_err(ProxyError::HttpError)?;
        
        Ok(response)
    }
}

fn backend_timeout(backends: &HashMap<String, BackendState>, route: Option<&RouteConfig>) -> u64 {
    if let Some(route) = route {
        if let Some(backend) = backends.get(&route.backend) {
            return backend.config.timeout;
        }
    }
    
    30
}

fn copy_headers(src: &HeaderMap, dst: &mut HeaderMap) {
    for (name, value) in src.iter() {
        if name == "connection" || name == "keep-alive" || name == "proxy-authenticate" 
           || name == "proxy-authorization" || name == "te" || name == "trailers"
           || name == "transfer-encoding" || name == "upgrade" {
            continue;
        }
        
        dst.insert(name, value.clone());
    }
}

fn get_client_ip(req: &Request<Body>) -> Option<String> {
    req.headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .or_else(|| req.headers().get("X-Real-IP").and_then(|h| h.to_str().ok()))
        .map(String::from)
} 
