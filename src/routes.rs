/*
A Rust Site Engine
Copyright 2020-2024 Anthony Martinez

Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
http://opensource.org/licenses/MIT>, at your option. This file may not be
copied, modified, or distributed except according to those terms.
*/

//! Provides an HTTP route handler using [`Engine`] to serve content with [`routerify`].

use anyhow::anyhow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use axum::{
    body::Body,
    extract::{Path as axumPath, State},
    http::StatusCode,
    response::Response,
    routing::get,
    Router,
};
use log::{debug, error, info};

use crate::common::slugify;

use super::render::Engine;
use super::{Context, Error, Result};

/// Creates a [`Filter`] instance with a given [`Arc<Engine>`].
pub(crate) fn router(engine: Arc<Engine>) -> Router {
    debug!("Building site router");
    let router = Router::new()
        .route("/", get(index_handler))
        .route("/favicon.ico", get(favicon))
        .route("/rss.xml", get(rss_handler))
        .route("/static/{*fname}", get(static_assets))
        .route("/{topic}/ext/{*fname}", get(topic_assets))
        .route("/{topic}/posts/{post}", get(post_handler))
        .route("/{topic}", get(topic_handler))
        .with_state(engine);

    router
}

/// Returns the MIME type given by the user's config for a particular extension.
/// By default this returns "text/plain" if no value is found. This makes it
/// critical for users to set MIME types for any file they intend to serve that
/// is not capable of being rendered as plaintext.
fn mime_from_ext(ext: Option<&OsStr>, mime_map: &HashMap<String, String>) -> String {
    if let Some(e) = ext {
        let e_string = e.to_str().unwrap_or_default();
        if let Some(m) = mime_map.get(e_string) {
            return m.clone();
        }
    }

    String::from("text/plain")
}

/// Handler for "/"
async fn index_handler(State(engine): State<Arc<Engine>>) -> Response<Body> {
    info!("Handling request to '/'");
    topic_posts(engine.clone(), "main".to_owned())
        .await
        .unwrap_or_else(|err| server_error(StatusCode::INTERNAL_SERVER_ERROR, err))
}

/// Handler for "/rss"
async fn rss_handler(State(engine): State<Arc<Engine>>) -> Response<Body> {
    info!("Handling request to '/rss.xml'");
    match engine.rss().await {
        Ok(rss) => Response::builder()
            .header("content-type", "application/rss+xml")
            .body(Body::from(rss))
            .unwrap(),
        Err(err) => server_error(StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

/// Handler for "/:topic"
async fn topic_handler(
    axumPath(topic): axumPath<String>,
    State(engine): State<Arc<Engine>>,
) -> Response<Body> {
    info!("Handling request to '/{}'", &topic);
    let topic_slug = slugify(&topic);
    if !engine.topic_slugs.contains(&topic_slug) {
        return server_error(
            StatusCode::NOT_FOUND,
            anyhow!("Topic: {} was not found", topic),
        );
    }

    topic_posts(engine.clone(), topic_slug)
        .await
        .unwrap_or_else(|err| {
            println!("wtf {err}");
            server_error(StatusCode::INTERNAL_SERVER_ERROR, err)
        })
}

/// Called by topic_handler to dynamically generate topic pages
async fn topic_posts(engine: Arc<Engine>, topic_slug: String) -> Result<Response<Body>> {
    let output = engine
        .render_topic(&topic_slug)
        .await
        .with_context(|| format!("failed to render topic: {}", &topic_slug))?;

    let response = Response::builder()
        .header("content-type", "text/html")
        .body(Body::from(output))?;
    Ok(response)
}

/// Handler for "/static/*fname"
async fn static_assets(
    axumPath(fname): axumPath<String>,
    State(engine): State<Arc<Engine>>,
) -> Response<Body> {
    info!("Handling static asset: '/static/{}'", &fname);
    if fname
        .split("/")
        .collect::<Vec<&str>>()
        .iter()
        .any(|x| x.eq(&".") || x.eq(&".."))
    {
        return server_error(
            StatusCode::FORBIDDEN,
            anyhow!("Attempted use of . or .. paths"),
        );
    }

    let static_path = Path::new(&engine.app.docpaths.webroot)
        .join("static")
        .join(fname);
    match File::open(&static_path)
        .await
        .with_context(|| format!("failed to open '{}'", &static_path.display()))
    {
        Ok(mut f) => {
            let mut buf = Vec::new();
            match f
                .read_to_end(&mut buf)
                .await
                .context("failed to read buffer")
            {
                Ok(_) => Response::builder()
                    .header(
                        "content-type",
                        mime_from_ext(static_path.extension(), &engine.app.mime_types),
                    )
                    .body(Body::from(buf))
                    .unwrap_or_else(|err| {
                        server_error(StatusCode::INTERNAL_SERVER_ERROR, err.into())
                    }),
                Err(err) => server_error(StatusCode::INTERNAL_SERVER_ERROR, err),
            }
        }
        Err(err) => server_error(StatusCode::NOT_FOUND, err),
    }
}

/// Handler for "/favicon.ico"
async fn favicon(State(engine): State<Arc<Engine>>) -> Response<Body> {
    info!("Handling favicon request");
    let favicon_path = Path::new(&engine.app.docpaths.webroot)
        .join("static")
        .join("favicon.ico");
    match File::open(&favicon_path).await {
        Ok(mut f) => {
            let mut buf = Vec::new();
            match f.read_to_end(&mut buf).await {
                Ok(_) => Response::builder()
                    .header("content-type", "image/vnd.microsoft.icon")
                    .body(Body::from(buf))
                    .unwrap_or_else(|err| {
                        server_error(StatusCode::INTERNAL_SERVER_ERROR, err.into())
                    }),
                Err(err) => server_error(StatusCode::INTERNAL_SERVER_ERROR, err.into()),
            }
        }
        Err(err) => server_error(StatusCode::INTERNAL_SERVER_ERROR, err.into()),
    }
}

/// Handler for "/:topic/ext/*fname"
async fn topic_assets(
    axumPath((topic, fname)): axumPath<(String, String)>,
    State(engine): State<Arc<Engine>>,
) -> Response<Body> {
    info!("Handling static asset: '/{}/ext/{}'", &topic, &fname);
    let topic_slug = slugify(&topic);
    if !engine.topic_slugs.contains(&topic_slug) {
        return server_error(
            StatusCode::NOT_FOUND,
            anyhow!("Topic: {} was not found", topic),
        );
    }

    if fname
        .split("/")
        .collect::<Vec<&str>>()
        .iter()
        .any(|x| x.eq(&".") || x.eq(&".."))
    {
        return server_error(
            StatusCode::FORBIDDEN,
            anyhow!("Attempted use of . or .. paths"),
        );
    }

    let topic_asset_path = Path::new(&engine.app.docpaths.webroot)
        .join(topic)
        .join("ext")
        .join(fname);

    match File::open(&topic_asset_path)
        .await
        .with_context(|| format!("failed to open '{}'", &topic_asset_path.display()))
    {
        Ok(mut f) => {
            let mut buf = Vec::new();
            match f
                .read_to_end(&mut buf)
                .await
                .context("failed to read buffer")
            {
                Ok(_) => Response::builder()
                    .header(
                        "content-type",
                        mime_from_ext(topic_asset_path.extension(), &engine.app.mime_types),
                    )
                    .body(Body::from(buf))
                    .unwrap_or_else(|err| {
                        server_error(StatusCode::INTERNAL_SERVER_ERROR, err.into())
                    }),
                Err(err) => server_error(StatusCode::INTERNAL_SERVER_ERROR, err),
            }
        }
        Err(err) => server_error(StatusCode::NOT_FOUND, err),
    }
}

/// Handler for "/:topic/posts/:post"
async fn post_handler(
    axumPath((topic, post)): axumPath<(String, String)>,
    State(engine): State<Arc<Engine>>,
) -> Response<Body> {
    info!("Handling topic post: '/{}/posts/{}'", &topic, &post);
    match engine
        .render_post(&slugify(&topic), &post)
        .await
        .with_context(|| format!("failed to render: '{}/posts/{}'", topic, post))
    {
        Ok(output) => Response::builder()
            .header("content-type", "text/html")
            .body(Body::from(output))
            .unwrap_or_else(|err| server_error(StatusCode::INTERNAL_SERVER_ERROR, err.into())),
        Err(err) => server_error(StatusCode::NOT_FOUND, err),
    }
}

/// Builds server error responses and logs originating error
fn server_error(code: StatusCode, err: Error) -> Response<Body> {
    error!("Server error: {err}");
    Response::builder()
        .status(code)
        .body(Body::default())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::prelude::*;

    use super::*;
    use crate::config::AppConfig;
    use hyper::StatusCode;
    use reqwest::Client;
    use tokio::sync::oneshot::channel;

    #[tokio::test]
    async fn check_all_handlers() {
        let dir = tempfile::tempdir().unwrap();
        let mut src: &[u8] = b"Site Name\nAuthor Name\nhttps://some.special.site\nOne, Two, Three, And More\nadmin\n";
        let app = AppConfig::generate(&dir, &mut src).unwrap();
        let engine = Engine::new(app);
        let engine = Arc::new(engine);

        let index_page = r#"
### Main Page

Main Important Test
"#;
        let mut f = File::create(dir.path().join("site/webroot/main/posts/index.md")).unwrap();
        f.write_all(index_page.as_bytes()).unwrap();

        let topic_page = r#"
### One Section

One Important Test
"#;
        let mut f = File::create(dir.path().join("site/webroot/one/posts/index.md")).unwrap();
        f.write_all(topic_page.as_bytes()).unwrap();

        let topic_asset = b"One Static File\n";

        let mut f = File::create(dir.path().join("site/webroot/one/ext/one-static")).unwrap();
        f.write_all(topic_asset).unwrap();

        let static_asset = b"Static File\n";

        let mut f = File::create(dir.path().join("site/webroot/static/main-static")).unwrap();
        f.write_all(static_asset).unwrap();

        let favicon = b"Favicon File\n";

        let mut f = File::create(dir.path().join("site/webroot/static/favicon.ico")).unwrap();
        f.write_all(favicon).unwrap();

        let router = router(engine.clone());
        let addr = format!("{}:{}", engine.app.server.bind, engine.app.server.port);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        let server = axum::serve(listener, router);

        let (tx, rx) = channel::<()>();

        let graceful = server.with_graceful_shutdown(async {
            rx.await.ok();
        });

        tokio::spawn(async move {
            if let Err(e) = graceful.await {
                println!("Encountered error: {}", e)
            }
        });

        let index_request_url = "http://localhost:9090";
        let post_request_url = "http://localhost:9090/one/posts/index";
        let topic_request_url = "http://localhost:9090/one";
        let topic_asset_request_url = "http://localhost:9090/one/ext/one-static";
        let static_asset_request_url = "http://localhost:9090/static/main-static";
        let favicon_request_url = "http://localhost:9090/favicon.ico";
        let bad_topic_request_url = "http://localhost:9090/badtopic";
        let bad_post_request_url = "http://localhost:9090/one/posts/nope";
        let bad_static_request_url = "http://localhost:9090/static/nope";
        let rss_request_url = "http://localhost:9090/rss.xml";

        let client = Client::new();

        let index_resp = client.get(index_request_url).send().await.unwrap();
        let post_resp = client.get(post_request_url).send().await.unwrap();
        let topic_resp = client.get(topic_request_url).send().await.unwrap();
        let topic_asset_resp = client.get(topic_asset_request_url).send().await.unwrap();
        let static_asset_resp = client.get(static_asset_request_url).send().await.unwrap();
        let favicon_resp = client.get(favicon_request_url).send().await.unwrap();
        let rss_resp = client.get(rss_request_url).send().await.unwrap();
        assert_eq!(index_resp.status(), StatusCode::OK);
        assert_eq!(post_resp.status(), StatusCode::OK);
        assert_eq!(topic_resp.status(), StatusCode::OK);
        assert_eq!(topic_asset_resp.status(), StatusCode::OK);
        assert_eq!(static_asset_resp.status(), StatusCode::OK);
        assert_eq!(favicon_resp.status(), StatusCode::OK);
        assert_eq!(rss_resp.status(), StatusCode::OK);

        let bad_topic_resp = client.get(bad_topic_request_url).send().await.unwrap();
        let bad_post_resp = client.get(bad_post_request_url).send().await.unwrap();
        let bad_static_resp = client.get(bad_static_request_url).send().await.unwrap();
        assert_eq!(bad_topic_resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(bad_post_resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(bad_static_resp.status(), StatusCode::NOT_FOUND);

        let _ = tx.send(());
    }

    #[tokio::test]
    async fn check_custom_config() {
        let app = AppConfig::from_path("test_files/test-config.toml").unwrap();
        let engine = Engine::new(app);
        let engine = Arc::new(engine);

        let router = router(engine.clone());

        let index_request_url = "http://localhost:8901";
        let post_request_url = "http://localhost:8901/one/posts/1";
        let topic_request_url = "http://localhost:8901/one";
        let gallery_request_url = "http://localhost:8901/gallery";
        let rss_request_url = "http://localhost:8901/rss.xml";

        let addr = format!("{}:{}", engine.app.server.bind, engine.app.server.port);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        let server = axum::serve(listener, router);

        let (tx, rx) = channel::<()>();

        let graceful = server.with_graceful_shutdown(async {
            rx.await.ok();
        });

        tokio::spawn(async move {
            if let Err(e) = graceful.await {
                println!("Encountered error: {}", e)
            }
        });

        let client = Client::new();

        let index_resp = client.get(index_request_url).send().await.unwrap();
        let post_resp = client.get(post_request_url).send().await.unwrap();
        let topic_resp = client.get(topic_request_url).send().await.unwrap();
        let gallery_resp = client.get(gallery_request_url).send().await.unwrap();
        let rss_resp = client.get(rss_request_url).send().await.unwrap();
        assert_eq!(index_resp.status(), StatusCode::OK);
        assert_eq!(post_resp.status(), StatusCode::OK);
        assert_eq!(topic_resp.status(), StatusCode::OK);
        assert_eq!(gallery_resp.status(), StatusCode::OK);
        assert_eq!(rss_resp.status(), StatusCode::OK);
        let _ = tx.send(());
    }
}
