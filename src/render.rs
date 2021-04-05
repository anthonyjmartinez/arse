use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::config::AppConfig;
use super::common;

pub use pulldown_cmark::{Parser, html};
pub use tera::{Tera, Context};

mod default;

pub struct Engine {
    pub app: Arc<AppConfig>,
    pub topic_slug: String,
    pub template: String,
    pub instance: Tera
}

impl Engine {
    pub fn new(a: Arc<AppConfig>, ts: &str, tmpl: &str, inst: Tera) -> Engine {
	Engine {
	    app: a.clone(),
	    topic_slug: ts.to_string(),
	    template: tmpl.to_string(),
	    instance: inst
	}
    }
}

pub fn load_default_template() -> Result<Tera, Box<dyn std::error::Error>> {
    let mut tera = Tera::default();
    tera.add_raw_template("default.tmpl", default::TEMPLATE)?;
    Ok(tera)
}

pub fn render_topic(engine: Engine) -> Result<String, Box<dyn std::error::Error>> {
    let blog = &engine.app.blog;
    let topic_data = load_topic(&engine)?;
    let mut context = Context::new();
    context.insert("blog", blog);
    context.insert("posts", &topic_data);
    let output = engine.instance.render(&engine.template, &context)?;

    Ok(output)
}

fn load_topic(engine: &Engine) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let topic_path = Path::new(&engine.app.docpaths.webroot).join(&engine.topic_slug);
    let pat = format!("{}/*.md", topic_path.display());
    let paths = common::path_matches(&pat)?;
    read_to_html(paths)
}

fn read_to_html(paths: Vec<PathBuf>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut contents: Vec<String> = Vec::new();
    for path in paths {
	let buf = std::fs::read_to_string(path)?;
	let parser = Parser::new(&buf);
	let mut html_output = String::new();
	html::push_html(&mut html_output, parser);
	contents.push(html_output);
    }

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use tempfile;

    #[test]
    fn check_default_template() {
	let tera = load_default_template();
	assert!(tera.is_ok())
    }

    #[test]
    fn check_render_topic() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let instance = load_default_template().unwrap();
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let engine = Engine::new(config, "one", "default.tmpl", instance);

	let post = r#"
### Something

Very cool, but maybe not super useful
"#;
	let post2 = r#"
### Something Again

Super Wow!
"#;
	
	let mut f = File::create(&dir.path().join("blog").join("webroot").join("one").join("post1.md")).unwrap();
	f.write_all(&post.as_bytes()).unwrap();

	let mut f = File::create(&dir.path().join("blog").join("webroot").join("one").join("post2.md")).unwrap();
	f.write_all(&post2.as_bytes()).unwrap();

	let page = render_topic(engine);

	assert!(page.is_ok())
    }
}
