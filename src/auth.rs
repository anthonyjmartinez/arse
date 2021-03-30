use std::io::prelude::*;

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
pub fn write_secret<T: Write>(secret: &str, dest: &mut T) -> std::io::Result<()> {
    dest.write_all(secret.as_bytes())
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
    fn check_write_secret() {
	let mut writer: Vec<u8> = vec![];
	const SECLEN: usize = 32;
	let secret = generate_secret(SECLEN).unwrap();
	write_secret(&secret, &mut writer).unwrap();
	assert_eq!(SECLEN, writer.len())
    }
}
