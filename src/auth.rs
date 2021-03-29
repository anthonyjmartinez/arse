/// TODO Document
pub fn generate_alphanum_password(len: usize) -> Result<String, Box<dyn std::error::Error>> {
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
