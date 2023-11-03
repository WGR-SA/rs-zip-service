use dotenv::dotenv;
use warp::Filter;

pub mod config;
pub mod handlers;
pub mod storage;

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load variables from `.env`

    println!("Hello, world!");

    let zip = warp::path("zip")
        .and(warp::body::json())
        .and(warp::header::header("X-Client"))
        .map(handlers::handle_compress_files);

    warp::serve(zip).run(([127, 0, 0, 1], 3030)).await;
}
