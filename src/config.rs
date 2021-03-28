use std::io::prelude::*;
use std::path::Path;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand, Values};
use serde::Deserialize;
use toml;


#[derive(Debug)]
pub enum Config {
    Main(AppConfig),
    New(BuildConfig),
}

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
		    .arg(Arg::with_name("username")
			 .help("Provides the username for the blog's admin")
			 .required(true)
			 .takes_value(true)
			 .short("u")
			 .long("user"))
		    .arg(Arg::with_name("topics")
			 .help("Provides the list of topics for the generator.")
			 .required(true)
			 .short("t")
			 .long("topic")
			 .multiple(true)
			 .min_values(1)))
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    let matches = args().get_matches();
    let config: Result<Config, Box<dyn std::error::Error>>;
    if matches.is_present("run") {
	config = runner_config(matches);
    } else if matches.is_present("new") {
	config = init_config(matches);
    } else {
	let msg = format!("Unable to load configuration");
	config = Err(From::from(msg));
    }

    config
}

fn runner_config(m: ArgMatches) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(run) = m.subcommand_matches("run") {
	let value = run.value_of("config").unwrap();
	let config = AppConfig::new(value)?;
	Ok(Config::Main(config))
    } else {
	let msg = format!("Failed to read arguments for 'run' subcommand");
	Err(From::from(msg))
    }
}

fn init_config(m: ArgMatches) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(new) = m.subcommand_matches("new") {
	let username = new.value_of("username").unwrap();
	let topic_values = new.values_of("topics").unwrap();
	let config = BuildConfig::new(&username, topic_values)?;
	Ok(Config::New(config))
    } else {
	let msg = format!("Failed to read arguments for 'new' subcommand");
	Err(From::from(msg))
    }
}

#[derive(Debug, Deserialize)]
pub struct Blog {
    name: String,
    author: String,
    topics: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    user: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct DocPaths {
    templates: String,
    webroot: String,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    level: String
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    blog: Blog,
    creds: Credentials,
    logging: LogConfig,
    docpaths: DocPaths,
}

impl AppConfig {
    pub fn new<T>(config: T) -> Result<AppConfig, Box<dyn std::error::Error>>
    where T: AsRef<Path> {
	let config = std::fs::read_to_string(config)?;
	let app_config: AppConfig = toml::from_str(&config)?;
	Ok(app_config)
    }
}

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    username: String,
    topics: Vec<String>
}

impl BuildConfig {
    pub fn new(username: &str, topic_values: Values) -> Result<BuildConfig, Box<dyn std::error::Error>> {
	let username = username.to_string();
	let topics: Vec<String> = topic_values.map(|x| String::from(x)).collect();
	Ok(BuildConfig { username, topics } )
    }

    pub fn to_app_config<R: BufRead>(&self, src: R) -> Result<AppConfig, Box<dyn std::error::Error>> {
	
	Err(From::from("just working on bits"))
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn build_run_config() {
	let arg_vec = vec!["caty-blog", "run", "./test_files/test-config.toml"];
	let matches = args().get_matches_from(arg_vec);
	let config = runner_config(matches);
	assert!(config.is_ok());
	let config = config.unwrap();
	assert!(matches!(config, Config::Main(_)));
    }

    #[test]
    fn build_new_config() {
	let arg_vec = vec!["caty-blog", "new", "-u", "caty",
			   "-t", "yoga", "cooking"];
	let matches = args().get_matches_from(arg_vec);
	let config = init_config(matches);
	assert!(config.is_ok());
	let config = config.unwrap();
	assert!(matches!(config, Config::New(_)));
    }

    #[test]
    // Fix this test to provide input for each "prompt" for information
    fn generate_blog_config() {
	let src: &[u8] = b"MagicPassword";
	let arg_vec = vec!["caty-blog", "new", "-u", "caty",
			   "-t", "yoga", "cooking"];
	let matches = args().get_matches_from(arg_vec);
	if let Ok(build_config) = init_config(matches) {
	    match build_config {
		Config::New(bc) => {
		    assert!(bc.to_app_config(src).is_ok())
		},
		_ => ()
	    }
	    
	}
    }
}
