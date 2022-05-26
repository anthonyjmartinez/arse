/*
A Rust Site Engine
Copyright 2020-2022 Anthony Martinez

Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
http://opensource.org/licenses/MIT>, at your option. This file may not be
copied, modified, or distributed except according to those terms.
*/

//! Main entry point for A Rust Site Engine.
//!
//! # Options
//! - `run [config]`: Starts a server defined by the `[config]` TOML.
//! - `new`: Creates a new `[config]` TOML from user input, and creates
//!          the site's directory structure.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{anyhow, Context, Error, Result};
use hyper::Server;
use log::{error, info};
use routerify::RouterService;

mod common;
mod config;
mod render;
mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::load()?;
    let app = Arc::new(config);
    info!("Configuration loaded");

    let engine = Arc::new(render::Engine::new(app));
    info!("Rendering Engine loaded");

    let router = routes::router(engine.clone());
    info!("Route handlers loaded");

    let service = RouterService::new(router).unwrap();
    info!("Service loaded");

    let addr = format!("{}:{}", engine.app.server.bind, engine.app.server.port);
    let addr: SocketAddr = addr.parse().unwrap();

    info!("Creating server on: {}", &addr);
    let server = Server::bind(&addr).serve(service);

    info!("Running server on: {}", &addr);
    if let Err(err) = server.await {
        error!("Server error: {}", err)
    }

    Ok(())
}
