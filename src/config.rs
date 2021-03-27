use std::collections::HashMap;
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
		    .about("Generates a base directory structure for a new blog")
		    .arg(Arg::with_name("topics")
			 .help("Provides the list of topics for the generator.")
			 .required(true)
			 .short("t")
			 .long("topic")
			 .multiple(true)
			 .min_values(1)))
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
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
	if let Some(value) = run.value_of("config") {
	    let config = AppConfig::new(value)?;
	    Ok(Config::Main(config))
	} else {
	    let msg = format!("Failed to load configuration");
	    Err(From::from(msg))
	}
    } else {
	let msg = format!("Failed to read arguments for 'run' subcommand");
	Err(From::from(msg))
    }
}

fn init_config(m: ArgMatches) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(new) = m.subcommand_matches("new") {
	if let Some(values) = new.values_of("topics") {
	    let config = BuildConfig::new(values)?;
	    Ok(Config::New(config))
	} else {
	    let msg = format!("Failed to load configuration");
	    Err(From::from(msg))
	}
    } else {
	let msg = format!("Failed to read arguments for 'new' subcommand");
	Err(From::from(msg))
    }
}

type Dict = HashMap<String, String>;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    creds: Dict,
    logging: Dict,
    docpaths: Dict,
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
    topics: Vec<String>
}

impl BuildConfig {
    pub fn new(values: Values) -> Result<BuildConfig, Box<dyn std::error::Error>> {
	let topics: Vec<String> = values.map(|x| String::from(x)).collect();
	Ok(BuildConfig { topics } )
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
	let arg_vec = vec!["caty-blog", "new", "-t", "yoga", "cooking"];
	let matches = args().get_matches_from(arg_vec);
	let config = init_config(matches);
	assert!(config.is_ok());
	let config = config.unwrap();
	assert!(matches!(config, Config::New(_)));
    }
}
