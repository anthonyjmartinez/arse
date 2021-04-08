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
    /*
    use std::fs::File;
    use std::io::prelude::*;
    use std::net::SocketAddr;

    use super::*;
    use crate::config::AppConfig;
    use routerify::RouterService;
    use tempfile;

    // TODO: figure out how to actually test this....
    
    #[tokio::test]
    async fn router_test() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let app = AppConfig::generate(&dir, &mut src).unwrap();
	let app = Arc::new(app);
	let router = router(app.clone());
	let service = RouterService::new(router).unwrap();
	let addr: SocketAddr = "0.0.0.0:9999".parse().unwrap();

    }
    */

}
