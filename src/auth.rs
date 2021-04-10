// A Rust Site Engine
// Copyright 2020-2021 Anthony Martinez
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

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
