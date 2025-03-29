use crate::{
    config::{BackendConfig, Config, RouteConfig},
    error::ProxyResult,
};
use hyper::{Body, Request, Response};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::debug;
pub struct ProxyService {
    config: Config,
    routes: Vec<RouteConfig>,
    backends: HashMap<String, BackendState>,
}

struct BackendState {
    config: BackendConfig,
    next_server_index: RwLock<usize>,
}

impl ProxyService {
    pub async fn proxy_request(&self, req: Request<Body>) -> ProxyResult<Response<Body>> {
        let path = req.uri().path();
        debug!("Received request for path: {}", path);

    }
}
