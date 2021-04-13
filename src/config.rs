/*
A Rust Site Engine
Copyright 2020-2021 Anthony Martinez

Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
http://opensource.org/licenses/MIT>, at your option. This file may not be
copied, modified, or distributed except according to those terms.
*/

use std::fs::create_dir_all;
use std::{io::BufRead, usize};
use std::path::Path;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand, crate_authors, crate_description, crate_version};
use log::{debug, error, info, trace};
use simplelog::{SimpleLogger, ConfigBuilder};
use serde::{Serialize, Deserialize};

use super::auth;
use super::common;

fn args() -> App<'static, 'static> {
    App::new("A Rust Site Engine")
	.version(crate_version!())
	.author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("verbosity")
             .short("v")
             .multiple(true) 
             .help("Sets the log level. Default: INFO. -v = DEBUG, -vv = TRACE"))
        .subcommand(SubCommand::with_name("run")
		    .about("Run the site server")
		    .arg(Arg::with_name("config")
			 .help("Provides the path to the server configuration file.")
			 .required(true)
			 .takes_value(true)
			 .index(1)))
	.subcommand(SubCommand::with_name("new")
		    .about("Generates a base directory structure and configuration file for a new site")
		    )
}

/// TODO Document this public function
/// And Include an Example of its Use
pub fn load() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config: Result<AppConfig, Box<dyn std::error::Error>>;
    let matches = args().get_matches();

    // Create a Config with ISO timestamps
    let log_config = ConfigBuilder::new()
        .set_time_format_str("%+")
        .build();

    // After this block locking is configured at the specified level
    match matches.occurrences_of("verbosity") {
	0 => SimpleLogger::init(log::LevelFilter::Info, log_config)?,
	1 => SimpleLogger::init(log::LevelFilter::Debug, log_config)?,
	_ => SimpleLogger::init(log::LevelFilter::Trace, log_config)?,
    }

    info!("Logging started");

    debug!("Processing subcommands");
    if matches.is_present("run") {
	trace!("Application called with `run` subcommand - loading config from disk");
	config = runner_config(matches);
    } else if matches.is_present("new") {
	trace!("Application called with `new` subcommand - creating config from user input");
	let reader = std::io::stdin();
	let mut reader = reader.lock();
	let current_path = std::env::current_dir()?;
	config = AppConfig::generate(current_path, &mut reader);
    } else {
	let msg = "Unable to load configuration".to_owned();
	error!("{}", &msg);
	config = Err(From::from(msg));
    }

    config
}

fn runner_config(m: ArgMatches) -> Result<AppConfig, Box<dyn std::error::Error>> {
    if let Some(run) = m.subcommand_matches("run") {
	let value = run.value_of("config").unwrap();
	let config = AppConfig::from_path(value)?;
	Ok(config)
    } else {
	let msg = "Failed to read arguments for 'run' subcommand".to_owned();
	error!("{}", &msg);
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
    debug!("Creating Vec<String> from csv topics: {}", &csv);
    let val_vec: Vec<String> = csv.split(',')
        .map(|s| s
             .trim_start_matches(char::is_whitespace)
             .trim_end_matches(char::is_whitespace)
             .to_string())
        .collect();

    val_vec
}

/// TODO Document
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Site {
    pub name: String,
    pub author: String,
    pub topics: Vec<String>,
}

impl Site {
    pub fn new_from_input<R: BufRead>(reader: &mut R) -> Result<Site, Box<dyn std::error::Error>> {
	let name = get_input("Please enter a name for the site: ", reader)?;
	let author = get_input("Please enter the site author's name: ", reader)?;
	let topics = get_input("Please enter comma-separated site topics: ", reader)?;
	let topics = csv_to_vec(&topics);
	let site = Site { name, author, topics };

	trace!("Site: {:?}", site);
	Ok(site)
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
	let user = get_input("Please enter an username for the site admin: ", reader)?;
	const PASSWORD_LEN: usize = 32;
	let password = auth::generate_secret(PASSWORD_LEN)?;
	info!("Site admin password generated");
	let password_file = dir.as_ref().join("admin.pass");
	common::str_to_ro_file(&password, password_file)?;
	let password = auth::generate_argon2_phc(&password)?;

	const TOKEN_LEN: usize = 34;
	let token = auth::generate_secret(TOKEN_LEN)?;
	let token = token.as_bytes();
	let token = auth::BASE32_NOPAD.encode(token);
	info!("Site admin TOTP token generated");
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
	debug!("Creating site DocPaths");
	let dir = dir.as_ref().display();
	let templates = format!("{}/site/templates", dir); 
	let webroot = format!("{}/site/webroot", dir); 
	let docpaths = DocPaths { templates, webroot };

	trace!("Site DocPaths: {:?}", docpaths);
	Ok(docpaths)
    }
}

/// TODO Document
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct AppConfig {
    pub site: Site,
    pub creds: Credentials,
    pub docpaths: DocPaths,
}

impl AppConfig {
    pub fn from_path<T: AsRef<Path>>(config: T) -> Result<AppConfig, Box<dyn std::error::Error>> {
	debug!("Loading site configuration from {}", &config.as_ref().display());
	let config = std::fs::read_to_string(config)?;

	trace!("Parsing configuration TOML");
	let app_config: AppConfig = toml::from_str(&config)?;

	Ok(app_config)
    }

    pub fn generate<P: AsRef<Path>, R: BufRead>(dir: P, reader: &mut R) -> Result<AppConfig, Box<dyn std::error::Error>> {
	debug!("Generating new site configuration");
	let docpaths = DocPaths::new(&dir)?;
	let site = Site::new_from_input(reader)?;
	let creds = Credentials::new_from_input(&dir, reader)?;
	
	let config = AppConfig {
	    site,
	    creds,
	    docpaths,
	};

	config.create_paths()?;
	config.write(&dir)?;

	Ok(config)
    }

    fn create_paths(&self) -> Result<(), Box<dyn std::error::Error>> {
	debug!("Creating site filesystem tree");
	create_dir_all(&self.docpaths.templates)?;
	create_dir_all(format!("{}/static/ext", &self.docpaths.webroot))?;
	create_dir_all(format!("{}/main/ext", &self.docpaths.webroot))?;
	create_dir_all(format!("{}/main/posts", &self.docpaths.webroot))?;

	for topic in &self.site.topics {
	    let topic = common::slugify(&topic);

	    create_dir_all(format!("{}/{}/ext", &self.docpaths.webroot, &topic))?;
	    create_dir_all(format!("{}/{}/posts", &self.docpaths.webroot, &topic))?;
	}
	Ok(())
    }

    fn write<P: AsRef<Path>>(&self, dir: P) -> Result<(), Box<dyn std::error::Error>> {
	debug!("Writing site configuration to disk");
	let config = toml::to_string_pretty(&self)?;
	let conf_path = &dir.as_ref().join("config.toml");
	common::str_to_ro_file(&config, &conf_path)?;
	Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_run_config() {
	let arg_vec = vec!["arse", "run", "./test_files/test-config.toml"];
	let matches = args().get_matches_from(arg_vec);
	let config = runner_config(matches);
	assert!(config.is_ok());
    }

    #[test]
    fn build_config_from_input() {
	let dir = tempfile::tempdir().unwrap();
	// Setup all target fields
	let mut src: &[u8] = b"Site Name\nAuthor Name\nOne, Two, Three, And More\nadmin\n";
	let config = AppConfig::generate(&dir, &mut src);
	assert!(config.is_ok());

	let tmp_dir = &dir.path();
	let config_path = &tmp_dir.join("config.toml");
	let admin = &tmp_dir.join("admin.pass");
	let token = &tmp_dir.join("admin.totp");
	let site = &tmp_dir.join("site");
	let templates = &tmp_dir.join("site/templates");
	let webroot = &tmp_dir.join("site/webroot");
	let static_ext = &tmp_dir.join("site/webroot/static/ext");
	let main_ext = &tmp_dir.join("site/webroot/main/ext");
	let main_posts = &tmp_dir.join("site/webroot/main/posts");
	let one_ext = &tmp_dir.join("site/webroot/one/ext");
	let one_posts = &tmp_dir.join("site/webroot/one/posts");
	let two_ext = &tmp_dir.join("site/webroot/two/ext");
	let two_posts = &tmp_dir.join("site/webroot/two/posts");
	let three_ext = &tmp_dir.join("site/webroot/three/ext");
	let three_posts = &tmp_dir.join("site/webroot/three/posts");
	let and_more_ext = &tmp_dir.join("site/webroot/and-more/ext");
	let and_more_posts = &tmp_dir.join("site/webroot/and-more/posts");
	let core = vec![config_path, admin, token, site, templates,
			webroot, static_ext, main_ext, main_posts,
			one_ext, one_posts, two_ext, two_posts,
			three_ext, three_posts, and_more_ext, and_more_posts];
	for p in core {
	    assert!(Path::new(p).exists())
	}
    }

    #[test]
    fn handle_csv_topics() {
	let reference_topics: Vec<String> = vec!["One".to_owned(),
						 "Two".to_owned(),
						 "Three".to_owned(),
						 "And More".to_owned()];
	let topics = "One, Two, Three, And More".to_owned();
	assert_eq!(reference_topics, csv_to_vec(&topics))
    }

}
