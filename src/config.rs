use std::fs::create_dir_all;
use std::{io::BufRead, usize};
use std::path::Path;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use serde::{Serialize, Deserialize};
use toml;

use crate::auth;
use crate::common;

fn args() -> App<'static, 'static> {
    App::new("Caty's Blog")
	.version("1.0")
	.author("Anthony Martinez")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("run")
		    .about("Run the blog server")
		    .arg(Arg::with_name("config")
			 .help("Provides the path to the server configuration file.")
			 .required(true)
			 .takes_value(true)
			 .index(1)))
	.subcommand(SubCommand::with_name("new")
		    .about("Generates a base directory structure and configuration file for a new blog")
		    )
}

/// TODO Document this public function
/// And Include an Example of its Use
pub fn load() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let matches = args().get_matches();
    let config: Result<AppConfig, Box<dyn std::error::Error>>;
    if matches.is_present("run") {
	config = runner_config(matches);
    } else if matches.is_present("new") {
	let reader = std::io::stdin();
	let mut reader = reader.lock();
	let current_path = std::env::current_dir()?;
	config = AppConfig::generate(current_path, &mut reader);
    } else {
	let msg = format!("Unable to load configuration");
	config = Err(From::from(msg));
    }

    config
}

fn runner_config(m: ArgMatches) -> Result<AppConfig, Box<dyn std::error::Error>> {
    if let Some(run) = m.subcommand_matches("run") {
	let value = run.value_of("config").unwrap();
	let config = AppConfig::new(value)?;
	Ok(config)
    } else {
	let msg = format!("Failed to read arguments for 'run' subcommand");
	Err(From::from(msg))
    }
}

fn get_input<R: BufRead>(prompt: &str, reader: &mut R) -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = String::new();
    println!("{}", prompt);
    reader.read_line(&mut buf)?;
    let buf = String::from(buf
			   .trim_start_matches(char::is_whitespace)
			   .trim_end_matches(char::is_whitespace));
    Ok(buf)
}

fn csv_to_vec(csv: &str) -> Vec<String> {
    let val_vec: Vec<String> = csv.split(",")
        .map(|s| s
             .trim_start_matches(char::is_whitespace)
             .trim_end_matches(char::is_whitespace)
             .to_string())
        .collect();

    val_vec
}

/// TODO Document
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Blog {
    pub name: String,
    pub author: String,
    pub topics: Vec<String>,
}

impl Blog {
    pub fn new_from_input<R: BufRead>(reader: &mut R) -> Result<Blog, Box<dyn std::error::Error>> {
	let name = get_input("Please enter a name for the blog: ", reader)?;
	let author = get_input("Please enter the blog author's name: ", reader)?;
	let topics = get_input("Please enter comma-separated blog topics: ", reader)?;
	let topics = csv_to_vec(&topics);
	let blog = Blog { name, author, topics };

	Ok(blog)
    }
}

/// TODO Document
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Credentials {
    pub user: String,
    pub password: String,
    pub token: String,
}

impl Credentials {
    pub fn new_from_input<P: AsRef<Path>, R: BufRead>(dir: P, reader: &mut R) -> Result<Credentials, Box<dyn std::error::Error>> {
	let user = get_input("Please enter an username for the blog admin: ", reader)?;
	const PASSWORD_LEN: usize = 32;
	let password = auth::generate_secret(PASSWORD_LEN)?;
	let password_file = dir.as_ref().join("admin.pass");
	common::str_to_ro_file(&password, password_file)?;
	let password = auth::generate_argon2_phc(&password)?;

	const TOKEN_LEN: usize = 34;
	let token = auth::generate_secret(TOKEN_LEN)?;
	let token = token.as_bytes();
	let token = auth::BASE32_NOPAD.encode(token);
	let token_file = dir.as_ref().join("admin.totp");
	common::str_to_ro_file(&token, token_file)?;
	
	let creds = Credentials { user, password, token };

	Ok(creds)
    }
}

/// TODO Document
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DocPaths {
    pub templates: String,
    pub webroot: String,
}

impl DocPaths {
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<DocPaths, Box<dyn std::error::Error>> {
	let dir = dir.as_ref().display();
	let templates = format!("{}/blog/templates", dir); 
	let webroot = format!("{}/blog/webroot", dir); 
	let docpaths = DocPaths { templates, webroot };

	Ok(docpaths)
    }
}

/// TODO Document
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LogConfig {
    level: String
}

/// TODO Document
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct AppConfig {
    pub blog: Blog,
    pub creds: Credentials,
    pub logging: LogConfig,
    pub docpaths: DocPaths,
}

impl AppConfig {
    pub fn new<T>(config: T) -> Result<AppConfig, Box<dyn std::error::Error>>
    where T: AsRef<Path> {
	let config = std::fs::read_to_string(config)?;
	let app_config: AppConfig = toml::from_str(&config)?;
	Ok(app_config)
    }

    pub fn generate<P: AsRef<Path>, R: BufRead>(dir: P, reader: &mut R) -> Result<AppConfig, Box<dyn std::error::Error>> {
	let docpaths = DocPaths::new(&dir)?;
	let blog = Blog::new_from_input(reader)?;
	let creds = Credentials::new_from_input(&dir, reader)?;
	let level = format!("INFO");
	let logging = LogConfig { level };
	
	let config = AppConfig {
	    blog,
	    creds,
	    logging,
	    docpaths,
	};

	config.create_paths()?;
	config.write(&dir)?;

	Ok(config)
    }

    fn create_paths(&self) -> Result<(), Box<dyn std::error::Error>> {
	create_dir_all(&self.docpaths.templates)?;
	create_dir_all(format!("{}/static/ext", &self.docpaths.webroot))?;
	create_dir_all(format!("{}/main/ext", &self.docpaths.webroot))?;
	create_dir_all(format!("{}/main/posts", &self.docpaths.webroot))?;

	for topic in &self.blog.topics {
	    let topic = common::slugify(&topic);

	    create_dir_all(format!("{}/{}/ext", &self.docpaths.webroot, &topic))?;
	    create_dir_all(format!("{}/{}/posts", &self.docpaths.webroot, &topic))?;
	}
	Ok(())
    }

    fn write<P: AsRef<Path>>(&self, dir: P) -> Result<(), Box<dyn std::error::Error>> {
	let config = toml::to_string_pretty(&self)?;
	let conf_path = &dir.as_ref().join("config.toml");
	common::str_to_ro_file(&config, &conf_path)?;
	Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn build_run_config() {
	let arg_vec = vec!["caty-blog", "run", "./test_files/test-config.toml"];
	let matches = args().get_matches_from(arg_vec);
	let config = runner_config(matches);
	assert!(config.is_ok());
    }

    #[test]
    fn build_config_from_input() {
	let dir = tempfile::tempdir().unwrap();
	// Setup all target fields
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src);
	assert!(config.is_ok());

	let tmp_dir = &dir.path();
	let config_path = &tmp_dir.join("config.toml");
	let admin = &tmp_dir.join("admin.pass");
	let token = &tmp_dir.join("admin.totp");
	let blog = &tmp_dir.join("blog");
	let templates = &tmp_dir.join("blog/templates");
	let webroot = &tmp_dir.join("blog/webroot");
	let static_ext = &tmp_dir.join("blog/webroot/static/ext");
	let main_ext = &tmp_dir.join("blog/webroot/main/ext");
	let main_posts = &tmp_dir.join("blog/webroot/main/posts");
	let one_ext = &tmp_dir.join("blog/webroot/one/ext");
	let one_posts = &tmp_dir.join("blog/webroot/one/posts");
	let two_ext = &tmp_dir.join("blog/webroot/two/ext");
	let two_posts = &tmp_dir.join("blog/webroot/two/posts");
	let three_ext = &tmp_dir.join("blog/webroot/three/ext");
	let three_posts = &tmp_dir.join("blog/webroot/three/posts");
	let and_more_ext = &tmp_dir.join("blog/webroot/and-more/ext");
	let and_more_posts = &tmp_dir.join("blog/webroot/and-more/posts");
	let core = vec![config_path, admin, token, blog, templates,
			webroot, static_ext, main_ext, main_posts,
			one_ext, one_posts, two_ext, two_posts,
			three_ext, three_posts, and_more_ext, and_more_posts];
	for p in core {
	    assert!(Path::new(p).exists())
	}
    }

    #[test]
    fn handle_csv_topics() {
	let reference_topics: Vec<String> = vec![format!("One"), format!("Two"), format!("Three"), format!("And More")];
	let topics = format!("One, Two, Three, And More");
	assert_eq!(reference_topics, csv_to_vec(&topics))
    }

}
