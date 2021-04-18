/*
A Rust Site Engine
Copyright 2020-2021 Anthony Martinez

Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
http://opensource.org/licenses/MIT>, at your option. This file may not be
copied, modified, or distributed except according to those terms.
*/

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{anyhow, Context, Error, Result};
use log::{info, error};
use hyper::Server;
use routerify::RouterService;

mod auth;
mod config;
mod common;
mod render;
mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::load()?;
    let app = Arc::new(config);
    info!("Configuration loaded");
    
    let router = routes::router(app.clone());
    info!("Route handlers loaded");

    let service = RouterService::new(router).unwrap();
    info!("Service loaded");

    let addr: SocketAddr = "0.0.0.0:9090".parse().unwrap();

    info!("Creating server on: {}", &addr);
    let server = Server::bind(&addr).serve(service);

    info!("Running server on: {}", &addr);
    if let Err(err) = server.await {
	error!("Server error: {}", err)
    }

    Ok(())
}
