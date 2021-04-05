use std::path::Path;
use std::sync::Arc;

use super::config::AppConfig;
use super::render;

pub fn topic_posts(app: Arc<AppConfig>, topic: String) { 
    let app = app.clone();
    let t = topic.clone();
    let handler = move || {
	let mut body = String::new();
	if let Ok(instance) = render::load_default_template() {
	    let engine = render::Engine::new(app.clone(), &t, "default.tmpl", instance);
	    if let Ok(output) = render::render_topic(engine) {
		body = output;
	    }
	}

    };

    if &topic == "main" {
	// Iron route
    } else {
	// Iron route
    }
}

pub fn static_assets(app: Arc<AppConfig>) { 
    let app = &app.clone();
    let static_path = Path::new(&app.docpaths.webroot).join("static");
    // Iron route
}

pub fn topic_assets(app: Arc<AppConfig>, topic: String) { 
    let app = &app.clone();
    let topic_asset_path = Path::new(&app.docpaths.webroot).join(&topic).join("ext");
    // Iron route
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::prelude::*;

    use super::*;
    use crate::config::AppConfig;
    use tempfile;

    #[tokio::test]
    async fn main_page() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let filter = topic_posts(config, "main".to_owned());

	let test_file = b"### Some title\n\nsome bytes here\n";
	let mut f = File::create(dir.path().join("blog/webroot/main/posts/1.md")).unwrap();
	f.write_all(test_file).unwrap();

    }

    #[tokio::test]
    async fn topic_page() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let filter = topic_posts(config, "and-more".to_owned());

	let test_file = b"### Some title\n\nsome bytes here\n";
	let mut f = File::create(dir.path().join("blog/webroot/and-more/posts/1.md")).unwrap();
	f.write_all(test_file).unwrap();

    }

    #[tokio::test]
    async fn static_content() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);

	let test_file = b"some bytes here\n";
	let mut f = File::create(dir.path().join("blog/webroot/static/example")).unwrap();
	f.write_all(test_file).unwrap();
	let filter = static_assets(config);

    }

    #[tokio::test]
    async fn topic_static() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);

	let test_file = b"some bytes here\n";
	let mut f = File::create(dir.path().join("blog/webroot/one/ext/example")).unwrap();
	f.write_all(test_file).unwrap();

	let filter = topic_assets(config.clone(), "one".to_owned());

    }
}
