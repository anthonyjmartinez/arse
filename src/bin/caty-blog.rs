use caty_blog::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config

    // Configure logging

    // Do shit based on arguments
    // - Either generate the base directory structure for a blog
    // - Or load an existing site and serve its routes based on the loaded config
    // warp::serve(routes).run(([127,0,0,1], 3030)).await;

    Ok(())
}
