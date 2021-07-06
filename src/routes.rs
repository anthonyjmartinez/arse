/*
A Rust Site Engine
Copyright 2020-2021 Anthony Martinez

Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
http://opensource.org/licenses/MIT>, at your option. This file may not be
copied, modified, or distributed except according to those terms.
*/

//! Provides an HTTP route handler using [`Engine`] to serve content with [`routerify`].

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use log::{debug, error, info};
use routerify::{prelude::*, Router, RouteError};

use super::render::Engine;
use super::{Context, Error, Result};


/// Creates a [`Router<Body, Error>`] instance with a given [`Arc<Engine>`].
pub(crate) fn router(engine: Arc<Engine>) -> Router<Body, Error> {
    debug!("Building site router");
    Router::builder()
        .data(engine)
        .get("/static/:fname", static_assets)
        .get("/favicon.ico", favicon)
        .get("/rss", rss_handler)
        .get("/:topic", topic_handler)
        .get("/:topic/ext/:fname", topic_assets)
        .get("/:topic/posts/:post", post_handler)
        .get("/", index_handler)
        .err_handler(error_handler)
        .build()
        .unwrap()
}

/// Handles errors from either bad requests or server errors
pub(crate) async fn error_handler(err: RouteError) -> Response<Body> {
    error!("{}", err);

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}

/// Handler for "/"
async fn index_handler(req: Request<Body>) -> Result<Response<Body>> {
    info!("Handling request to '/'");
    let engine = req.data::<Arc<Engine>>().unwrap();
    topic_posts(engine.clone(), "main".to_owned()).await
}

/// Handler for "/rss"
async fn rss_handler(req: Request<Body>) -> Result<Response<Body>> {
    let engine = req.data::<Arc<Engine>>().unwrap();
    info!("Handling request to '/rss'");
    // TODO: call a rss function on Engine

    Ok(Response::new(Body::from("TEMPORARY")))
}

/// Handler for "/:topic"
async fn topic_handler(req: Request<Body>) -> Result<Response<Body>> {
    let engine = req.data::<Arc<Engine>>().unwrap();
    let topic = req.param("topic").unwrap();
    info!("Handling request to '/{}'", &topic);
    topic_posts(engine.clone(), topic.to_owned()).await
}

/// Called by topic_handler to dynamically generate topic pages 
async fn topic_posts(engine: Arc<Engine>, topic: String) -> Result<Response<Body>> { 
    let output = engine.render_topic(&topic)
        .with_context(|| format!("failed to render topic: {}", &topic))?;
    Ok(Response::new(Body::from(output)))
}

/// Handler for "/static/:fname"
async fn static_assets(req: Request<Body>) -> Result<Response<Body>> { 
    let engine = req.data::<Arc<Engine>>().unwrap();
    let resource = req.param("fname").unwrap();
    info!("Handling static asset: '/static/{}'", &resource);
    let static_path = Path::new(&engine.app.docpaths.webroot).join("static").join(resource);
    let mut f = File::open(&static_path)
        .with_context(|| format!("failed to open '{}'", &static_path.display()))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).context("failed to read to buffer")?;
    Ok(Response::new(Body::from(buf)))
}

/// Handler for "/favicon.ico"
async fn favicon(req: Request<Body>) -> Result<Response<Body>> { 
    let engine = req.data::<Arc<Engine>>().unwrap();
    info!("Handling favicon request");
    let favicon_path = Path::new(&engine.app.docpaths.webroot).join("static").join("favicon.ico");
    let mut f = File::open(&favicon_path)
        .with_context(|| format!("failed to open '{}'", &favicon_path.display()))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).context("failed to read to buffer")?;
    Ok(Response::new(Body::from(buf)))
}

/// Handler for "/:topic/ext/:fname"
async fn topic_assets(req: Request<Body>) -> Result<Response<Body>> { 
    let engine = req.data::<Arc<Engine>>().unwrap();
    let topic = req.param("topic").unwrap();
    let resource = req.param("fname").unwrap();
    info!("Handling static asset: '/{}/ext/{}'", &topic, &resource);
    let topic_asset_path = Path::new(&engine.app.docpaths.webroot).join(topic).join("ext").join(resource);
    let mut f = File::open(&topic_asset_path)
        .with_context(|| format!("failed to open '{}'", &topic_asset_path.display()))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).context("failed to read to buffer")?;
    Ok(Response::new(Body::from(buf)))
}

/// Handler for "/:topic/:post"
async fn post_handler(req: Request<Body>) -> Result<Response<Body>> { 
    let engine = req.data::<Arc<Engine>>().unwrap();
    let topic = req.param("topic").unwrap();
    let post = req.param("post").unwrap();
    info!("Handling topic post: '/{}/posts/{}'", &topic, &post);
    let output = engine.render_post(topic, post)
        .with_context(|| format!("failed to render: '{}/posts/{}'", topic, post))?;
    Ok(Response::new(Body::from(output)))
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
    use tokio::sync::oneshot::channel;


    #[tokio::test]
    async fn check_all_handlers() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Site Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let app = AppConfig::generate(&dir, &mut src).unwrap();
	let engine = Engine::new(Arc::new(app));
	let engine = Arc::new(engine);

	let index_page = r#"
### Main Page

Main Important Test
"#;
	let mut f = File::create(&dir.path().join("site/webroot/main/posts/index.md")).unwrap();
	f.write_all(&index_page.as_bytes()).unwrap();

	let topic_page = r#"
### One Section

One Important Test
"#;
	let mut f = File::create(&dir.path().join("site/webroot/one/posts/index.md")).unwrap();
	f.write_all(&topic_page.as_bytes()).unwrap();

	let topic_asset = b"One Static File\n";

	let mut f = File::create(&dir.path().join("site/webroot/one/ext/one-static")).unwrap();
	f.write_all(topic_asset).unwrap();

	let static_asset = b"Static File\n";

	let mut f = File::create(&dir.path().join("site/webroot/static/main-static")).unwrap();
	f.write_all(static_asset).unwrap();

	let favicon = b"Favicon File\n";

	let mut f = File::create(&dir.path().join("site/webroot/static/favicon.ico")).unwrap();
	f.write_all(favicon).unwrap();

	let router = router(engine.clone());

	let index_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:9090")
	    .body(Body::default())
	    .unwrap();

	let post_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:9090/one/posts/index")
	    .body(Body::default())
	    .unwrap();
	
	let topic_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:9090/one")
	    .body(Body::default())
	    .unwrap();

	let topic_asset_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:9090/one/ext/one-static")
	    .body(Body::default())
	    .unwrap();

	let static_asset_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:9090/static/main-static")
	    .body(Body::default())
	    .unwrap();

	let favicon_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:9090/favicon.ico")
	    .body(Body::default())
	    .unwrap();

	let bad_topic_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:9090/badtopic")
	    .body(Body::default())
	    .unwrap();

	let bad_post_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:9090/one/posts/nope")
	    .body(Body::default())
	    .unwrap();

	let bad_static_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:9090/static/nope")
	    .body(Body::default())
	    .unwrap();

	let service = RouterService::new(router).unwrap();
	let addr = format!("{}:{}", engine.app.server.bind, engine.app.server.port);
	let addr: SocketAddr = addr.parse().unwrap();

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
	let post_resp = client.request(post_request).await.unwrap();
	let topic_resp = client.request(topic_request).await.unwrap();
	let topic_asset_resp = client.request(topic_asset_request).await.unwrap();
	let static_asset_resp = client.request(static_asset_request).await.unwrap();
	let favicon_resp = client.request(favicon_request).await.unwrap();
	assert_eq!(index_resp.status(), StatusCode::OK);
	assert_eq!(post_resp.status(), StatusCode::OK);
	assert_eq!(topic_resp.status(), StatusCode::OK);
	assert_eq!(topic_asset_resp.status(), StatusCode::OK);
	assert_eq!(static_asset_resp.status(), StatusCode::OK);
	assert_eq!(favicon_resp.status(), StatusCode::OK);

	let bad_topic_resp = client.request(bad_topic_request).await.unwrap();
	let bad_post_resp = client.request(bad_post_request).await.unwrap();
	let bad_static_resp = client.request(bad_static_request).await.unwrap();
	assert_eq!(bad_topic_resp.status(), StatusCode::NOT_FOUND);
	assert_eq!(bad_post_resp.status(), StatusCode::NOT_FOUND);
	assert_eq!(bad_static_resp.status(), StatusCode::NOT_FOUND);

	let _ = tx.send(());
    }

    #[tokio::test]
    async fn check_custom_config() {
	let app = AppConfig::from_path("test_files/test-config.toml").unwrap() ;
	let engine = Engine::new(Arc::new(app));
	let engine = Arc::new(engine);

	let router = router(engine.clone());

	let index_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:8901")
	    .body(Body::default())
	    .unwrap();

	let post_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:8901/one/posts/1")
	    .body(Body::default())
	    .unwrap();

	let topic_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:8901/one")
	    .body(Body::default())
	    .unwrap();

	let gallery_request = Request::builder()
	    .method("GET")
	    .uri("http://localhost:8901/gallery")
	    .body(Body::default())
	    .unwrap();

	let service = RouterService::new(router).unwrap();
	let addr = format!("{}:{}", engine.app.server.bind, engine.app.server.port);
	let addr: SocketAddr = addr.parse().unwrap();

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
	let post_resp = client.request(post_request).await.unwrap();
	let topic_resp = client.request(topic_request).await.unwrap();
	let gallery_resp = client.request(gallery_request).await.unwrap();
	assert_eq!(index_resp.status(), StatusCode::OK);
	assert_eq!(post_resp.status(), StatusCode::OK);
	assert_eq!(topic_resp.status(), StatusCode::OK);
	assert_eq!(gallery_resp.status(), StatusCode::OK);
	let _ = tx.send(());
    }

}
