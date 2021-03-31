use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;


/// TODO Document
pub fn generate_secret(len: usize) -> Result<String, Box<dyn std::error::Error>> {
    use rand::{thread_rng, Rng,
	    distributions::Alphanumeric};

    if len < 32 {
	Err(From::from("Random passwords shorter than 32ch are useless"))
    } else {
	let pass: String = thread_rng()
	    .sample_iter(&Alphanumeric)
	    .take(len)
	    .map(char::from)
	    .collect();
	Ok(pass)
    }
}

/// TODO Document
pub fn generate_argon2_phc(secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    use rand::rngs::OsRng;
    use argon2::{Argon2, password_hash::{SaltString, PasswordHasher}};

    let secret = secret.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let argon2_phc: Result<String, Box<dyn std::error::Error>>;
    if let Ok(phc) = argon2.hash_password_simple(secret, salt.as_ref()) {
	argon2_phc = Ok(phc.to_string());
    } else {
	argon2_phc = Err(From::from("Failed to hash password"));
    }

    argon2_phc
}

#[cfg(target_family = "unix")]
pub fn write_secret_file<P: AsRef<Path>>(secret: &str, dest: P) -> std::io::Result<()> { 
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = OpenOptions::new();
    options.create(true);
    options.write(true);
    options.mode(0o600);
    let mut secret_file = options.open(dest)?;
    secret_file.write_all(secret.as_bytes())
}

#[cfg(target_family = "windows")]
pub fn write_secret_file<P: AsRef<Path>>(secret: &str, dest: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut secret_file = File::create(dest)?;
    secret_file.write_all(secret.as_bytes())?;
    let metadata = secret_file.metadata()?;
    let mut perms = metadata.permissions();
    perms.set_readonly(true);
    Ok(())
}


pub use data_encoding::BASE32_NOPAD;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_secret_len() {
	const SECLEN: usize = 32;
	let secret = generate_secret(SECLEN).unwrap();
	assert_eq!(SECLEN, secret.len())
    }

    #[test]
    fn check_short_secret() {
	const SECLEN: usize = 12;
	let secret = generate_secret(SECLEN);
	assert!(secret.is_err())
    }

    #[test]
    fn check_argon2_hasher() {
	const SECLEN: usize = 32;
	let secret = generate_secret(SECLEN).unwrap();
	let phc = generate_argon2_phc(&secret);
	assert!(phc.is_ok())
    }
}
