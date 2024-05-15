/*
A Rust Site Engine
Copyright 2020-2024 Anthony Martinez

Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
http://opensource.org/licenses/MIT>, at your option. This file may not be
copied, modified, or distributed except according to those terms.
*/

//! Provides the rendering engine for topics and posts using [`AppConfig`], [`Tera`], and [`pulldown_cmark`].

use tokio::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::common;
use super::config::AppConfig;
use super::{Context, Result};

use chrono::{DateTime, Utc};
use log::{debug, trace};
use pulldown_cmark::{html, Parser};
use rss::{Channel, Item};
use tera::{Context as TemplateContext, Tera};

/// Static defaults for the rendering engine.
mod default;

/// Rendering engine for topics and posts.
///
/// [`Engine`] stores an [`Arc<AppConfig>`] and a [`Tera`] instance from which
/// all rendering and serving tasks are executed.
#[derive(Debug)]
pub(crate) struct Engine {
    pub app: Arc<AppConfig>,
    pub instance: Tera,
}

impl Engine {
    /// Creates a new [`Engine`] from a given [`AppConfig`].
    pub(crate) fn new(app: Arc<AppConfig>) -> Engine {
	trace!("Loading rendering engine");
	let instance = Self::load_template(app.clone()).unwrap();
	Engine { app, instance }
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

    /// Renders `/:topic` content as HTML
    pub(crate) async fn render_topic(&self, topic_slug: &str) -> Result<String> {
	let site = &self.app.site;
	let mut context = TemplateContext::new();
	context.insert("site", site);

	if topic_slug == "gallery" {
	    debug!("Rendering image gallery");
	    // Need to make this an async call
	    let gallery = self.load_gallery()?;
	    context.insert("gallery", &gallery);
	} else {
	    debug!("Rendering topic: '{}'", topic_slug);
	    // Need to make this an async call
	    let topic_data = self.load_topic(topic_slug).await?;
	    context.insert("posts", &topic_data);
	}

	let output = self
	    .instance
	    .render(&site.template, &context)
	    .with_context(|| {
		format!(
		    "failed rendering topic: {}, with Tera instance: {:?}",
		    topic_slug, self.instance
		)
	    })?;

	trace!("Rendered content for topic: {}\n{}", topic_slug, output);
	Ok(output)
    }

    async fn load_topic(&self, topic_slug: &str) -> Result<Vec<String>> {
	trace!("Loading topic content for '{}'", topic_slug);
	let topic_path = Path::new(&self.app.docpaths.webroot)
	    .join(topic_slug)
	    .join("posts");
	let pat = format!("{}/*.md", topic_path.display());
	let paths = common::path_matches(&pat)?;
	Self::read_all_to_html(paths).await
    }

    async fn read_all_to_html(paths: Vec<PathBuf>) -> Result<Vec<String>> {
	debug!("Rendering Topic Markdown to HTML");
	let mut contents: Vec<String> = Vec::new();
	for path in paths {
	    trace!("Rendering {} to HTML", &path.display());
	    let buf = tokio::fs::read_to_string(&path)
		.await
		.with_context(|| format!("failure reading '{}' to string", &path.display()))?;
	    let parser = Parser::new(&buf);
	    let mut html_output = String::new();
	    html::push_html(&mut html_output, parser);
	    contents.push(html_output);
	}

	Ok(contents)
    }

    fn load_gallery(&self) -> Result<Vec<PathBuf>> {
	debug!("Loading gallery content");
	let gallery_path = Path::new(&self.app.docpaths.webroot)
	    .join("gallery")
	    .join("ext");
	let pat = format!("{}/*.jpg", gallery_path.display());
	let mut paths = common::path_matches(&pat)?;

	trace!("Stripping path prefixes");
	for path in paths.iter_mut() {
	    *path = path.strip_prefix(&self.app.docpaths.webroot)?.to_path_buf();
	}

	trace!("Gallery items: {:?}", paths);
	Ok(paths)
    }

    /// Renders `/:topic/posts/:post` content as HTML
    pub(crate) async fn render_post(&self, topic_slug: &str, post: &str) -> Result<String> {
	debug!("Rendering post: '{}'", post);
	let site = &self.app.site;
	let post_data = self.load_post(topic_slug, post).await?;
	let mut context = TemplateContext::new();
	context.insert("site", site);
	context.insert("post", &post_data);
	let output = self
	    .instance
	    .render(&site.template, &context)
	    .with_context(|| {
		format!(
		    "failed rendering topic: {}, with Tera instance: {:?}",
		    topic_slug, self.instance
		)
	    })?;

	trace!("Rendered content for post: {}\n{}", topic_slug, output);
	Ok(output)
    }

    async fn load_post(&self, topic_slug: &str, post: &str) -> Result<String> {
	trace!("Loading post content for '{}'", post);
	let topic_path = Path::new(&self.app.docpaths.webroot)
	    .join(topic_slug)
	    .join("posts");
	let post_path = format!("{}/{}.md", topic_path.display(), post);
	Self::read_post_to_html(post_path).await
    }

    async fn read_post_to_html<P: AsRef<Path>>(path: P) -> Result<String> {
	debug!("Rendering Post Markdown to HTML");
	trace!("Rendering {} to HTML", &path.as_ref().display());
	let buf = tokio::fs::read_to_string(&path)
	    .await
	    .with_context(|| format!("failure reading '{}' to string", &path.as_ref().display()))?;
	let parser = Parser::new(&buf);
	let mut html_output = String::new();
	html::push_html(&mut html_output, parser);

	Ok(html_output)
    }

    /// Renders `/rss.xml` for all topics
    pub(crate) async fn rss(&self) -> Result<String> {
	debug!("Rendering RSS Feed");
	let site = &self.app.site;
	let items = Self::rss_items(self).await?;
	let mut channel = Channel::default();
	channel.set_title(&site.name);
	channel.set_link(&site.url);
	channel.set_description(format!("{} RSS Feed", &site.name));
	channel.set_items(items);

	Ok(channel.to_string())
    }

    async fn rss_items(&self) -> Result<Vec<Item>> {
	debug!("Building RSS Items");
	let mut items: Vec<Item> = Vec::new();
	items.append(&mut Self::topic_to_item(self, "main").await?);

	for topic in &self.app.site.topics {
	    let mut topic_items = Self::topic_to_item(self, &common::slugify(topic)).await?;
	    items.append(&mut topic_items);
	}

	Ok(items)
    }

    async fn topic_to_item(&self, topic_slug: &str) -> Result<Vec<Item>> {
	trace!("Generating RSS Items for topic: {}", &topic_slug);
	let mut items: Vec<Item> = Vec::new();
	let topic_path = Path::new(&self.app.docpaths.webroot)
	    .join(topic_slug)
	    .join("posts");
	let pat = format!("{}/*.md", topic_path.display());
	let paths = common::path_matches(&pat)?;
	for path in paths {
	    trace!("Generating RSS Item for post at: {}", path.display());
	    let link = format!(
		"{}/{}/{}",
		&self.app.site.url,
		path.strip_prefix(&self.app.docpaths.webroot)?
		    .parent()
		    .unwrap()
		    .to_str()
		    .unwrap(),
		path.file_stem().unwrap().to_str().unwrap()
	    );
	    let f = File::open(&path).await?;
	    let updated: DateTime<Utc> = f.metadata().await?.modified()?.into();

	    let updated = updated.to_rfc2822();

	    let description = Self::read_post_to_html(path).await?;

	    let mut item = Item::default();
	    item.set_link(link);
	    item.set_pub_date(updated);
	    item.set_description(description.to_owned());
	    items.push(item);
	}

	Ok(items)
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
	let mut src: &[u8] =
	    b"Site Name\nAuthor Name\nhttps://special.example.site\nOne, Gallery\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let tera = Engine::load_template(config);
	assert!(tera.is_ok())
    }

    #[tokio::test]
    async fn check_render_post() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] =
	    b"Site Name\nAuthor Name\nhttps://special.example.site\nOne, Gallery\nadmin\n";
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

	let mut f = File::create(dir.path().join("site/webroot/one/posts/post1.md")).unwrap();
	f.write_all(post.as_bytes()).unwrap();

	let mut f = File::create(dir.path().join("site/webroot/one/posts/post2.md")).unwrap();
	f.write_all(post2.as_bytes()).unwrap();

	let page1 = engine.render_post("one", "post1").await.unwrap();
	let page2 = engine.render_post("one", "post2").await.unwrap();

	assert!(page1.contains("super useful"));
	assert!(page2.contains("Super Wow!"));
    }

    #[tokio::test]
    async fn check_render_topic() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] =
	    b"Site Name\nAuthor Name\nhttps://special.example.site\nOne, Gallery\nadmin\n";
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

	let mut f = File::create(dir.path().join("site/webroot/one/posts/post1.md")).unwrap();
	f.write_all(post.as_bytes()).unwrap();

	let mut f = File::create(dir.path().join("site/webroot/one/posts/post2.md")).unwrap();
	f.write_all(post2.as_bytes()).unwrap();

	let page = engine.render_topic("one").await.unwrap();

	assert!(page.contains("super useful"));
	assert!(page.contains("Super Wow!"));
    }

    #[tokio::test]
    async fn check_render_empty_topic() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] =
	    b"Site Name\nAuthor Name\nhttps://special.example.site\nOne, Gallery\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let engine = Engine::new(config);

	let page = engine.render_topic("one").await.unwrap();

	assert!(page.contains("Coming Soon"));
    }

    #[tokio::test]
    async fn check_render_gallery_topic() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] =
	    b"Site Name\nAuthor Name\nhttps://special.example.site\nOne, Gallery\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let engine = Engine::new(config);

	let fake_img = "some bytes";
	let fake_img_2 = "some more bytes";

	let mut f = File::create(dir.path().join("site/webroot/gallery/ext/0.jpg")).unwrap();
	f.write_all(fake_img.as_bytes()).unwrap();

	let mut f = File::create(dir.path().join("site/webroot/gallery/ext/1.jpg")).unwrap();
	f.write_all(fake_img_2.as_bytes()).unwrap();
	let page = engine.render_topic("gallery").await.unwrap();

	assert!(page.contains("<script>"));
    }

    #[tokio::test]
    async fn check_render_empty_gallery() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] =
	    b"Site Name\nAuthor Name\nhttps://special.example.site\nOne, Gallery\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let engine = Engine::new(config);

	let page = engine.render_topic("gallery").await.unwrap();

	assert!(page.contains("Coming Soon"));
    }

    #[tokio::test]
    async fn check_render_rss() {
	let dir = tempfile::tempdir().unwrap();
	let mut src: &[u8] =
	    b"Site Name\nAuthor Name\nhttps://special.example.site\nOne, Gallery\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src).unwrap();
	let config = Arc::new(config);
	let engine = Engine::new(config);

	let main_post = r#"
### The Main Page

This is just a main topic page.
"#;
	let one_post1 = r#"
### A first post in One

Super Wow!
"#;
	let one_post2 = r#"
### [A second post in One](/one/posts/2)

Super Wow TWICE!
"#;

	let mut f = File::create(dir.path().join("site/webroot/main/posts/index.md")).unwrap();
	f.write_all(main_post.as_bytes()).unwrap();

	let mut f = File::create(dir.path().join("site/webroot/one/posts/1.md")).unwrap();
	f.write_all(one_post1.as_bytes()).unwrap();

	let mut f = File::create(dir.path().join("site/webroot/one/posts/2.md")).unwrap();
	f.write_all(one_post2.as_bytes()).unwrap();

	let rss = engine.rss().await.unwrap();

	assert!(rss.contains("The Main Page"));
	assert!(rss.contains("Super Wow!"));
	assert!(rss.contains("A second post in One"));
    }
}
