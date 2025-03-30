use std::convert::Infallible;
use std::sync::Arc;

use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::config::Config;
use crate::proxy::{ProxyService, create_proxy_service};

pub async fn run(config: Config) -> Result<()> {
    let addr = config.server.listen_addr;
    
    // Create shared proxy service
    let proxy_service = create_proxy_service(config.clone());
    let proxy_service = Arc::new(RwLock::new(proxy_service));
    
    // Create service function
    let make_svc = make_service_fn(move |_conn| {
        let proxy_service = proxy_service.clone();
        
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let proxy_service = proxy_service.clone();
                
                async move {
                    handle_request(proxy_service, req).await
                }
            }))
        }
    });
    
    // Create server
    let server = Server::bind(&addr).serve(make_svc);
    
    info!("Reverse proxy listening on http://{}", addr);
    
    // Run the server
    if let Err(e) = server.await {
        error!("Server error: {}", e);
        anyhow::bail!("Server error: {}", e);
    }
    
    Ok(())
}

async fn handle_request(
    proxy_service: Arc<RwLock<ProxyService>>,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    match proxy_service.read().await.proxy_request(req).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Error handling request: {}", e);
            
            // Return a 500 error response
            let response = Response::builder()
                .status(500)
                .body(Body::from(format!("Proxy error: {}", e)))
                .unwrap();
            
            Ok(response)
        }
    }
} 
