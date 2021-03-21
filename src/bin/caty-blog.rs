use caty_blog::*;
use warp::Filter;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Load configuration file defining paths to stuff
    // Load into memory as a Struct in an Arc(Mutex(something))
    // This should also include the admin username/hash(password)

    write_main(&render_main())?;

    let static_files = warp::path("static")
        .and(warp::fs::dir("webroot/static"));

    let index = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("webroot/index.html"));

    let routes = static_files.or(index);

    warp::serve(routes).run(([127,0,0,1], 3030)).await;

    Ok(())
}
