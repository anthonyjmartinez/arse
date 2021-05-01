/*
A Rust Site Engine
Copyright 2020-2021 Anthony Martinez

Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
http://opensource.org/licenses/MIT>, at your option. This file may not be
copied, modified, or distributed except according to those terms.
*/

use std::io::prelude::*;
use std::path::{Path, PathBuf};

use glob::glob;
use log::{debug, trace};

use super::{Context, Result};

#[cfg(target_family = "unix")]
pub(crate) fn str_to_ro_file<P: AsRef<Path>>(content: &str, dest: P) -> Result<()> { 
    debug!("Writing protected file: {}", &dest.as_ref().display());
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = OpenOptions::new();
    options.create(true);
    options.write(true);
    options.mode(0o600);
    
    trace!("Opening '{}' to write", &dest.as_ref().display());
    let mut ro_file = options.open(&dest)
	.with_context(|| format!("failed to open '{}' for writing", &dest.as_ref().display()))?;
    ro_file.write_all(content.as_bytes())
        .with_context(|| format!("failure writing '{}'", &dest.as_ref().display()))?;
    if !content.ends_with('\n') {
	ro_file.write_all(b"\n")
        .with_context(|| format!("failure writing '{}'", &dest.as_ref().display()))?;
    }
    trace!("Content written to destination");
    Ok(())
}

#[cfg(target_family = "windows")]
pub fn str_to_ro_file<P: AsRef<Path>>(content: &str, dest: P) -> Result<()> {
    debug!("Writing protected file: {}", &dest.as_ref().display());
    trace!("Opening '{}' to write", &dest.as_ref().display());
    let mut ro_file = std::fs::File::create(&dest)
        .with_context(|| format!("failed to open '{}' for writing", &dest.as_ref().display()))?;
    ro_file.write_all(content.as_bytes()).
	with_context(|| format!("failure writing '{}'", &dest.as_ref().display()))?;
    let metadata = ro_file.metadata()
        .with_context(|| format!("failure retrieving metadata on '{}'", &dest.as_ref().display()))?;
    let mut perms = metadata.permissions();
    if !content.ends_with('\n') {
	ro_file.write_all(b"\n")
	    .with_context(|| format!("failure writing '{}'", &dest.as_ref().display()))?;
    }
    trace!("Content written to destination");
    trace!("Setting read-only on destination file");
    perms.set_readonly(true);
    Ok(())
}

pub fn path_matches(pat: &str) -> Result<Vec<PathBuf>> {
    debug!("Building topic content vector from {}", &pat);
    let mut path_vec: Vec<PathBuf> = Vec::new();

    trace!("Globbing {}", &pat);
    let entries = glob(pat)
        .context("failure globbing paths")?;

    for entry in entries.filter_map(Result::ok) {
	trace!("Adding '{}' to topic content vector", &entry.display());
	path_vec.push(entry);
    }
    
    trace!("Reversing topic content vector for LIFO site rendering");
    path_vec.reverse();
    Ok(path_vec)
}

pub fn slugify(topic: &str) -> String {
    debug!("Creating slugified topic string from {}", &topic);
    topic.to_ascii_lowercase().replace(char::is_whitespace, "-")
}

