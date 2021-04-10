// A Rust Site Engine
// Copyright 2020-2021 Anthony Martinez
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::convert::Infallible;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;

use hyper::{Body, Request, Response};//, StatusCode};
use routerify::{prelude::*, Router};

use super::config::AppConfig;
use super::render;


pub fn router(app: Arc<AppConfig>) -> Router<Body, Infallible> {
    Router::builder()
        .data(app.clone())
        .get("/", index_handler)
        .get("/:topic", topic_handler)
        .get("/:topic/ext/:fname", topic_assets)
        .get("/static/:fname", static_assets)
        .build()
        .unwrap()
}

/// Handler for "/"
async fn index_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let app = req.data::<Arc<AppConfig>>().unwrap();
    topic_posts(app.clone(), "main".to_owned()).await
}

/// Handler for "/:topic"
async fn topic_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let app = req.data::<Arc<AppConfig>>().unwrap();
    let topic = req.param("topic").unwrap();
    topic_posts(app.clone(), topic.to_owned()).await
}

/// Called by topic_handler to dynamically generate topic pages 
async fn topic_posts(app: Arc<AppConfig>, topic: String) -> Result<Response<Body>, Infallible> { 
    let instance = render::load_default_template().unwrap();
    let engine = render::Engine::new(app.clone(), &topic, "default.tmpl", instance);
    let output = render::render_topic(engine).unwrap();
    Ok(Response::new(Body::from(output)))
}

/// Handler for "/static/:fname"
async fn static_assets(req: Request<Body>) -> Result<Response<Body>, Infallible> { 
    let app = req.data::<Arc<AppConfig>>().unwrap();
    let resource = req.param("fname").unwrap();
    let static_path = Path::new(&app.docpaths.webroot).join("static").join(resource);
    let mut f = File::open(static_path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    Ok(Response::new(Body::from(buf)))
}

/// Handler for "/:topic/ext/:fname"
async fn topic_assets(req: Request<Body>) -> Result<Response<Body>, Infallible> { 
    let app = req.data::<Arc<AppConfig>>().unwrap();
    let topic = req.param("topic").unwrap();
    let resource = req.param("fname").unwrap();
    let topic_asset_path = Path::new(&app.docpaths.webroot).join(topic).join("ext").join(resource);
    let mut f = File::open(topic_asset_path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    Ok(Response::new(Body::from(buf)))
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::net::SocketAddr;
    use std::io::prelude::*;

    use super::*;
    use crate::config::AppConfig;
    use hyper::{Body, Request, StatusCode, Server, Client};
    use routerify::RouterService;
    use tempfile;
    use tokio::sync::oneshot::channel;


    #[tokio::test]
    async fn check_all_handlers() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let app = AppConfig::generate(&dir, &mut src).unwrap();
	let app = Arc::new(app);
	
	let index_page = r#"
### Main Page

Main Important Test
"#;
	let mut f = File::create(&dir.path().join("blog/webroot/main/posts/index.md")).unwrap();
	f.write_all(&index_page.as_bytes()).unwrap();

	let topic_page = r#"
### One Section

One Important Test
"#;
	let mut f = File::create(&dir.path().join("blog/webroot/one/posts/index.md")).unwrap();
	f.write_all(&topic_page.as_bytes()).unwrap();

	let topic_asset = b"One Static File\n";

	let mut f = File::create(&dir.path().join("blog/webroot/one/ext/one-static")).unwrap();
	f.write_all(topic_asset).unwrap();

	let static_asset = b"Static File\n";

	let mut f = File::create(&dir.path().join("blog/webroot/static/main-static")).unwrap();
	f.write_all(static_asset).unwrap();

	let router = router(app.clone());

	let index_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:10000")
	    .body(Body::default())
	    .unwrap();

	let topic_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:10000/one")
	    .body(Body::default())
	    .unwrap();

	let topic_asset_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:10000/one/ext/one-static")
	    .body(Body::default())
	    .unwrap();

	let static_asset_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:10000/static/main-static")
	    .body(Body::default())
	    .unwrap();

	let service = RouterService::new(router).unwrap();
	let addr: SocketAddr = "127.0.0.1:10000".parse().unwrap();

	let (tx, rx) = channel::<()>();

	let server = Server::bind(&addr).serve(service);
	
	let graceful = server.
	    with_graceful_shutdown(async {
		rx.await.ok();
	    });

	tokio::spawn(async move {
	    if let Err(e) = graceful.await {
		println!("Encountered error: {}", e)
	    }
	});


	let client = Client::new();

	let index_resp = client.request(index_request).await.unwrap();
	let topic_resp = client.request(topic_request).await.unwrap();
	let topic_asset_resp = client.request(topic_asset_request).await.unwrap();
	let static_asset_resp = client.request(static_asset_request).await.unwrap();
	assert_eq!(index_resp.status(), StatusCode::OK);
	assert_eq!(topic_resp.status(), StatusCode::OK);
	assert_eq!(topic_asset_resp.status(), StatusCode::OK);
	assert_eq!(static_asset_resp.status(), StatusCode::OK);

	let _ = tx.send(());
    }
}
