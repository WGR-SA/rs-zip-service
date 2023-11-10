use actix_web::{get, http::header::ContentType, post, web, App, Error, HttpResponse, HttpServer};
use dotenv::dotenv;
use futures::{future::ok, stream::once};

pub mod config;
pub mod handlers;
pub mod storage;

#[get("/stream")]
async fn stream() -> HttpResponse {
    println!("stream");
    let body = once(ok::<_, Error>(web::Bytes::from_static(b"stream")));

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .streaming(body)
}

#[post("/zip")]
async fn zip(req: HttpRequest) -> HttpResponse {
    handlers::handle_compress_files(req)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load variables from `.env`
    println!("hello");
    HttpServer::new(|| App::new().service(zip).service(stream))
        .bind(("127.0.0.1", 3030))?
        .run()
        .await
}
