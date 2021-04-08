use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use glob::glob;

#[cfg(target_family = "unix")]
pub fn str_to_ro_file<P: AsRef<Path>>(content: &str, dest: P) -> Result<(), Box<dyn std::error::Error>> { 
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = OpenOptions::new();
    options.create(true);
    options.write(true);
    options.mode(0o600);
    let mut ro_file = options.open(dest)?;
    ro_file.write_all(content.as_bytes())?;
    if !content.ends_with("\n") {
	ro_file.write(b"\n")?;
    }
    Ok(())
}

#[cfg(target_family = "windows")]
pub fn str_to_ro_file<P: AsRef<Path>>(content: &str, dest: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut ro_file = File::create(dest)?;
    ro_file.write_all(content.as_bytes())?;
    let metadata = secret_file.metadata()?;
    let mut perms = metadata.permissions();
    if !content.ends_with("\n") {
	ro_file.write(b"\n")?;
    }
    perms.set_readonly(true);
    Ok(())
}

pub fn path_matches(pat: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut path_vec: Vec<PathBuf> = Vec::new();
    let entries = glob(pat)?;
    for entry in entries.filter_map(Result::ok) {
	path_vec.push(entry);
    }
    
    path_vec.reverse();
    Ok(path_vec)
}

pub fn slugify(topic: &str) -> String {
    let topic = topic
        .to_ascii_lowercase()
        .replace(char::is_whitespace, "-");
    topic
}

