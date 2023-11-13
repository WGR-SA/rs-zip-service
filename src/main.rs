pub mod config;
pub mod storage;
pub mod zip_stream;

use actix_web::{post, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use dotenv::dotenv;
use futures::{future::ok, stream::once};
use tokio::sync::mpsc;
use zip_stream::ZipStream;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load variables from `.env`
    println!("hello");
    HttpServer::new(|| App::new().service(get_files_stream_zip))
        .bind(("127.0.0.1", 3030))?
        .run()
        .await
}

#[post("/zip")]
async fn get_files_stream_zip(req: HttpRequest, body: web::Json<Vec<String>>) -> HttpResponse {
    let client = req.headers().get("X-Client").unwrap().to_str().unwrap();
    let config: config::StorageConfig = config::get_config(client).unwrap();

    let storage_instance = storage::get_storage(config);
    if !storage_instance.connect().await {
        return HttpResponse::InternalServerError().body("Error connecting to storage");
    }
    let (sender, receiver) = mpsc::channel(32);
    let (sender_test, mut receiver_test) = mpsc::channel(32);
    let stream = ZipStream::new(receiver);

    for path in body.into_inner() {
        println!("Sending file: {}", path);
        let _s = storage_instance.send_file_stream(&sender_test, path).await;
    }

    while let Some(file_stream) = receiver_test.recv().await {
        println!("Receving file: {}", file_stream.name);
    }

    HttpResponse::Ok()
        .content_type("application/zip")
        .append_header((
            "ContentDisposition",
            "attachment; filename=\"download.zip\"",
        ))
        .streaming(stream)
}
