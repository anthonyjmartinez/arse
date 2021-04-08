use std::net::SocketAddr;
use std::sync::Arc;

use caty_blog::*;
use hyper::Server;
use routerify::RouterService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load()?;
    let app = Arc::new(config);
    
    // TODO: Configure logging

    let router = routes::router(app.clone());
    let service = RouterService::new(router).unwrap();
    let addr: SocketAddr = "0.0.0.0:9090".parse().unwrap();
    let server = Server::bind(&addr).serve(service);

    if let Err(err) = server.await {
	eprintln!("Server error: {}", err)
    }

    Ok(())
}
