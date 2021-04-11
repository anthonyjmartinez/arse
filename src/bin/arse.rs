// A Rust Site Engine
// Copyright 2020-2021 Anthony Martinez
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::net::SocketAddr;
use std::sync::Arc;

use arse::*;
use log::{info, error};
use hyper::Server;
use routerify::RouterService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load()?;
    info!("Configuration loaded");
    let app = Arc::new(config);
    
    let router = routes::router(app.clone());
    let service = RouterService::new(router).unwrap();
    let addr: SocketAddr = "0.0.0.0:9090".parse().unwrap();
    let server = Server::bind(&addr).serve(service);

    info!("Running server on: {}", &addr);
    if let Err(err) = server.await {
	error!("Server error: {}", err)
    }

    Ok(())
}
