use std::path::Path;
use std::sync::Arc;
use warp::{Filter, Reply, filters::BoxedFilter};

use crate::config::AppConfig;

// Placeholder for Warp routes
/*
fn root(app: Arc<AppConfig>) -> BoxedFilter<(impl Reply,)> { 
    let app = &app.clone();
    let main_path = Path::new(&app.docpaths.webroot).join("main");



}
*/

fn static_assets(app: Arc<AppConfig>) -> BoxedFilter<(impl Reply,)> { 
    let app = &app.clone();
    let static_path = Path::new(&app.docpaths.webroot).join("static");

    warp::path("static")
        .and(warp::fs::dir(static_path))
	.boxed()
}

fn topic_assets(app: Arc<AppConfig>, topic: &'static str) -> BoxedFilter<(impl Reply,)> { 
    let app = &app.clone();
    let topic_asset_path = Path::new(&app.docpaths.webroot).join(topic).join("ext");
    warp::path(topic)
        .and(warp::path("ext"))
        .and(warp::fs::dir(topic_asset_path))
        .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use tempfile;

    /*
    #[tokio::test]
    async fn main_page() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let filter = root(config);

	assert!(!warp::test::request()
		.path("/")
		.matches(&filter)
		.await);

    }
    */

    #[tokio::test]
    async fn static_content() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let filter = static_assets(config);

	assert!(!warp::test::request()
		.path("/static/style.css")
		.matches(&filter)
		.await);

    }

    /*
    #[tokio::test]
    async fn topic_page() {
	let filter = topic();
    }
    */

    #[tokio::test]
    async fn topic_content() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let filter = topic_assets(config.clone(), "one");
	let filter2 = topic_assets(config.clone(), "two");
	let filter3 = topic_assets(config.clone(), "three");
	let filter_and_more = topic_assets(config.clone(), "and-more");

	assert!(!warp::test::request()
		.path("/one/ext/example.png")
		.matches(&filter)
		.await);

	assert!(!warp::test::request()
		.path("/two/ext/example.png")
		.matches(&filter2)
		.await);

	assert!(!warp::test::request()
		.path("/three/ext/example.png")
		.matches(&filter3)
		.await);

	assert!(!warp::test::request()
		.path("/and-more/ext/example.png")
		.matches(&filter_and_more)
		.await);
    }
}
