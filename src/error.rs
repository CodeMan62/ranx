use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] hyper::Error),
    
    #[error("TLS error: {0}")]
    TlsError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Backend error: {0}")]
    BackendError(String),
    
    #[error("No healthy backends available")]
    NoHealthyBackends,
    
    #[error("Route not found for path: {0}")]
    RouteNotFound(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,
}

impl From<rustls::Error> for ProxyError {
    fn from(err: rustls::Error) -> Self {
        ProxyError::TlsError(err.to_string())
    }
}

pub type ProxyResult<T> = std::result::Result<T, ProxyError>; 
