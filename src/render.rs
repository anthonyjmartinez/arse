/*
A Rust Site Engine
Copyright 2020-2021 Anthony Martinez

Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
http://opensource.org/licenses/MIT>, at your option. This file may not be
copied, modified, or distributed except according to those terms.
*/

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::config::AppConfig;
use super::common;
use super::{Context, Result};

use log::{debug, trace};
use pulldown_cmark::{Parser, html};
use tera::Tera;
use tera::Context as TemplateContext;

mod default;

#[derive(Debug)]
pub(crate) struct Engine {
    pub app: Arc<AppConfig>,
    pub instance: Tera,
}

impl Engine {
    pub(crate) fn new(app: Arc<AppConfig>) -> Engine {
	trace!("Loading rendering engine");
	let instance = Self::load_template(app.clone()).unwrap();
	Engine {
	    app,
	    instance,
	}
    }

    fn load_template(app: Arc<AppConfig>) -> Result<Tera> {
	trace!("Loading Tera rendering template");
	let mut tera = Tera::default();
	let template = app.site.template.as_str();
	let template_dir = PathBuf::from(&app.docpaths.templates);

	if let "default.tmpl" = template {
	    tera.add_raw_template("default.tmpl", default::TEMPLATE)
		.context("failure adding default template")?;
	} else {
	    let template_path = template_dir.join(template);
	    tera.add_template_file(template_path, Some(template))
		.context("failure loading template from file")?;
	}

	trace!("Tera template loaded: {:?}", tera);
	Ok(tera)
    }

    pub(crate) fn render_topic(&self, topic_slug: &str) -> Result<String> {
	debug!("Rendering topic: '{}'", topic_slug);
	let site = &self.app.site;
	let topic_data = self.load_topic(topic_slug)?;
	let mut context = TemplateContext::new();
	context.insert("site", site);
	context.insert("posts", &topic_data);
	let output = self.instance.render(&site.template, &context)
	    .with_context(|| format!("failed rendering topic: {}, with Tera instance: {:?}", topic_slug, self.instance))?;

	trace!("Rendered content for topic: {}\n{}", topic_slug, output);
	Ok(output)
    }

    fn load_topic(&self, topic_slug: &str) -> Result<Vec<String>> {
	trace!("Loading topic content for '{}'", topic_slug);
	let topic_path = Path::new(&self.app.docpaths.webroot).join(topic_slug).join("posts");
	let pat = format!("{}/*.md", topic_path.display());
	let paths = common::path_matches(&pat)?;
	Self::read_all_to_html(paths)
    }

    fn read_all_to_html(paths: Vec<PathBuf>) -> Result<Vec<String>> {
	debug!("Rendering Topic Markdown to HTML");
	let mut contents: Vec<String> = Vec::new();
	for path in paths {
	    trace!("Rendering {} to HTML", &path.display());
	    let buf = std::fs::read_to_string(&path)
		.with_context(|| format!("failure reading '{}' to string", &path.display()))?;
	    let parser = Parser::new(&buf);
	    let mut html_output = String::new();
	    html::push_html(&mut html_output, parser);
	    contents.push(html_output);
	}

	Ok(contents)
    }

    pub(crate) fn render_post(&self, topic_slug: &str, post: &str) -> Result<String> {
	debug!("Rendering post: '{}'", post);
	let site = &self.app.site;
	let post_data = self.load_post(topic_slug, post)?;
	let mut context = TemplateContext::new();
	context.insert("site", site);
	context.insert("post", &post_data);
	let output = self.instance.render(&site.template, &context)
	    .with_context(|| format!("failed rendering topic: {}, with Tera instance: {:?}", topic_slug, self.instance))?;

	trace!("Rendered content for post: {}\n{}", topic_slug, output);
	Ok(output)
    }

    fn load_post(&self, topic_slug: &str, post: &str) -> Result<String> {
	trace!("Loading post content for '{}'", post);
	let topic_path = Path::new(&self.app.docpaths.webroot).join(topic_slug).join("posts");
	let post_path = format!("{}/{}.md", topic_path.display(), post);
	Self::read_post_to_html(post_path)
    }


    fn read_post_to_html<P: AsRef<Path>>(path: P) -> Result<String> {
	debug!("Rendering Post Markdown to HTML");
	trace!("Rendering {} to HTML", &path.as_ref().display());
	let buf = std::fs::read_to_string(&path)
	    .with_context(|| format!("failure reading '{}' to string", &path.as_ref().display()))?;
	let parser = Parser::new(&buf);
	let mut html_output = String::new();
	html::push_html(&mut html_output, parser);

	Ok(html_output)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn check_default_template() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Site Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let tera = Engine::load_template(config);
	assert!(tera.is_ok())
    }

    #[test]
    fn check_render_post() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Site Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let engine = Engine::new(config);

	let post = r#"
### Something

Very cool, but maybe not super useful
"#;
	let post2 = r#"
### Something Again

Super Wow!
"#;
	
	let mut f = File::create(&dir.path().join("site/webroot/one/posts/post1.md")).unwrap();
	f.write_all(&post.as_bytes()).unwrap();

	let mut f = File::create(&dir.path().join("site/webroot/one/posts/post2.md")).unwrap();
	f.write_all(&post2.as_bytes()).unwrap();

	let page1 = engine.render_post("one", "post1").unwrap();
	let page2 = engine.render_post("one", "post2").unwrap();

	assert!(page1.contains("super useful"));
	assert!(page2.contains("Super Wow!"));
    }

    #[test]
    fn check_render_topic() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] = b"Site Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let engine = Engine::new(config);

	let post = r#"
### Something

Very cool, but maybe not super useful
"#;
	let post2 = r#"
### Something Again

Super Wow!
"#;
	
	let mut f = File::create(&dir.path().join("site/webroot/one/posts/post1.md")).unwrap();
	f.write_all(&post.as_bytes()).unwrap();

	let mut f = File::create(&dir.path().join("site/webroot/one/posts/post2.md")).unwrap();
	f.write_all(&post2.as_bytes()).unwrap();

	let page = engine.render_topic("one").unwrap();

	assert!(page.contains("super useful"));
	assert!(page.contains("Super Wow!"));
    }
}
