use warp::Filter;

#[tokio::main]
async fn main() {
    let zip = warp::path("zip")
        .and(warp::body::json())
        .map(create_and_serve_zip);

    warp::serve(zip).run(([127, 0, 0, 1], 3030)).await;
}

fn create_and_serve_zip(body: Vec<String>) -> String {
    format!("{:?}", body)
}
