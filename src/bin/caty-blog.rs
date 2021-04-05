use std::sync::Arc;

use caty_blog::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load()?;
    let app = Arc::new(config);

    // Configure logging

    Ok(())
}
