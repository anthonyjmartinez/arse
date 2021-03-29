use std::{io::BufRead, usize};
use std::path::Path;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use serde::Deserialize;
use toml;

use crate::auth;

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
	config = init_config();
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

fn init_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let reader = std::io::stdin();
    let mut reader = reader.lock();
    let config = AppConfig::generate(&mut reader)?;
    Ok(config)
}

/// TODO Document
#[derive(Debug, Deserialize, PartialEq)]
pub struct Blog {
    name: String,
    author: String,
    topics: Vec<String>,
}

/// TODO Document
#[derive(Debug, Deserialize, PartialEq)]
pub struct Credentials {
    user: String,
    password: String,
}

/// TODO Document
#[derive(Debug, Deserialize, PartialEq)]
pub struct DocPaths {
    templates: String,
    webroot: String,
}

/// TODO Document
#[derive(Debug, Deserialize, PartialEq)]
pub struct LogConfig {
    level: String
}

/// TODO Document
#[derive(Debug, Deserialize, PartialEq)]
pub struct AppConfig {
    blog: Blog,
    creds: Credentials,
    logging: LogConfig,
    docpaths: DocPaths,
}

impl AppConfig {
    fn new<T>(config: T) -> Result<AppConfig, Box<dyn std::error::Error>>
    where T: AsRef<Path> {
	let config = std::fs::read_to_string(config)?;
	let app_config: AppConfig = toml::from_str(&config)?;
	Ok(app_config)
    }

    fn generate<R: BufRead>(reader: &mut R) -> Result<AppConfig, Box<dyn std::error::Error>> {
	let current_path = std::env::current_dir()?;
	let current_path = current_path.display();

	let name = Self::get_input("Please enter a name for the blog: ", reader)?;
	let author = Self::get_input("Please enter the blog author's name: ", reader)?;
	let topics = Self::get_input("Please enter comma-separated blog topics: ", reader)?;
	let topics: Vec<String> = topics.split(",")
	    .map(|s| s
		 .trim_start_matches(char::is_whitespace)
		 .trim_end_matches(char::is_whitespace)
		 .to_string())
	    .collect();
	let blog = Blog { name, author, topics };

	let user = Self::get_input("Please enter an username for the blog admin: ", reader)?;

	const PASSWORD_LEN: usize = 32;
	let password = auth::generate_alphanum_password(PASSWORD_LEN)?;
	println!("Save this random password for your admin: {}", password);

	let creds = Credentials { user, password };

	let templates = format!("{}/{}/templates", current_path, "blog"); 
	let webroot = format!("{}/{}/webroot", current_path, "blog"); 
	let docpaths = DocPaths { templates, webroot };

	let level = format!("INFO");
	let logging = LogConfig { level };
	
	let config = AppConfig {
	    blog,
	    creds,
	    logging,
	    docpaths,
	};

	Ok(config)
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
    }

    #[test]
    fn generate_blog_config() {
	// Setup all target fields
	let name = format!("Blog Name");
	let author = format!("Author Name");
	let topics: Vec<String> = vec![format!("One"), format!("Two"), format!("Three"), format!("And More")];
	let blog = Blog { name, author, topics };

	let user = format!("admin");
	let password = format!("MagicPassword");
	let creds = Credentials { user, password };

	let current_path = std::env::current_dir().unwrap();
	let current_path = current_path.display();
	let templates = format!("{}/blog/templates", current_path);
	let webroot = format!("{}/blog/webroot", current_path);
	let docpaths = DocPaths { templates, webroot };

	let level = format!("INFO");
	let logging = LogConfig { level };
	
	let reference_config = AppConfig {
	    blog,
	    creds,
	    logging,
	    docpaths,
	};
	
	let mut src: &[u8] = b"Blog Name\nAuthor Name\nOne, Two, Three, And More\nadmin\nMagicPassword\nINFO\n";
	let arg_vec = vec!["caty-blog", "new"];
	let matches = args().get_matches_from(arg_vec);
	if  matches.is_present("new") {
	    if let Ok(config) = generate(&mut src) {
		assert_eq!(reference_config, config)
	    } else {
		panic!("Failed to generate the reference config")
	    }
	}
    }

    // Generate function locally for testing AppConfig::get_input
    fn generate<R: BufRead>(reader: &mut R) -> Result<AppConfig, Box<dyn std::error::Error>> {
	let current_path = std::env::current_dir()?;
	let current_path = current_path.display();

	let name = AppConfig::get_input("Please enter a name for the blog: ", reader)?;
	let author = AppConfig::get_input("Please enter the blog author's name: ", reader)?;
	let topics = AppConfig::get_input("Please enter comma-separated blog topics: ", reader)?;
	let topics: Vec<String> = topics.split(",")
	    .map(|s| s
		 .trim_start_matches(char::is_whitespace)
		 .trim_end_matches(char::is_whitespace)
		 .to_string())
	    .collect();
	let blog = Blog { name, author, topics };

	let user = AppConfig::get_input("Please enter an username for the blog admin: ", reader)?;
	let password = AppConfig::get_input("Please enter a password for the blog admin: ", reader)?;
	let creds = Credentials { user, password };

	let templates = format!("{}/{}/templates", current_path, "blog"); 
	let webroot = format!("{}/{}/webroot", current_path, "blog"); 
	let docpaths = DocPaths { templates, webroot };

	let level = format!("INFO");
	let logging = LogConfig { level };
	
	let config = AppConfig {
	    blog,
	    creds,
	    logging,
	    docpaths,
	};

	Ok(config)
    }
}
