use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use pulldown_cmark::{Parser, html};
use tera::{Tera, Context};

// Placeholder for Warp routes

/*
pub fn read_to_html(paths: Vec<PathBuf>) -> Vec<String> {
    let mut contents: Vec<String> = Vec::new();
    for path in paths {
	let buffer = std::fs::read_to_string(path).unwrap();
	let parser = Parser::new(&buffer);
	let mut html_output = String::new();
	html::push_html(&mut html_output, parser);
	contents.push(html_output);
    }
    contents
}

pub fn load_templates(dir: &str) -> Tera {
    match Tera::new(dir) {
	Ok(t) => t,
	Err(e) => panic!("Failed with {}", e)
    }
}

pub fn render_main() -> String {
    let pattern = parse_args();
    let path_vec = path_matches(&pattern);
    let mut data = read_to_html(path_vec);
    data.reverse();

    let tera = load_templates("templates/*.tmpl");
    let mut context = Context::new();
    
    context.insert("posts", &data);

    let output = tera.render("main.tmpl", &context);

    if let Ok(output) = output {
	output
    } else {
	String::new()
    }
}

pub fn write_main(rendered: &str) -> Result<(), Box<dyn std::error::Error>> {
    let buf = rendered.as_bytes();
    let mut f = File::create("webroot/index.html")?;
    f.write_all(buf)?;
    Ok(())
}
*/
