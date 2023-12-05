pub mod config;
pub mod storage;
pub mod zip;

use actix_web::{
    http::header::ContentDisposition, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use async_zip::base::write::ZipFileWriter;
use async_zip::Compression;
use async_zip::ZipEntryBuilder;
use async_zip::ZipString;
use dotenv::dotenv;
use futures::io::AsyncWriteExt;
use futures::stream::StreamExt;
use std::env;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load variables from `.env`

    HttpServer::new(|| App::new().service(get_files_stream_zip))
        .bind((
            "127.0.0.1",
            env::var("PORT").unwrap().parse::<u16>().unwrap(),
        ))?
        .run()
        .await
}

#[post("/zip")]
async fn get_files_stream_zip(req: HttpRequest, body: web::Json<Vec<String>>) -> HttpResponse {
    let client = req.headers().get("X-Client").unwrap().to_str().unwrap();
    let config: config::StorageConfig = match config::get_config(client) {
        Ok(config) => config,
        Err(_) => return HttpResponse::InternalServerError().body("Error getting config"),
    };

    let storage_instance = storage::get_storage(config);
    if !storage_instance.connect().await {
        return HttpResponse::InternalServerError().body("Error connecting to storage");
    }

    let (input_sender, mut input_receiver) = mpsc::channel(200);
    let (output_sender, output_receiver) = mpsc::channel::<bytes::Bytes>(10000);

    let response_stream = ReceiverStream::new(output_receiver).map(Ok::<_, Error>);

    let writer = zip::writer::ZipWriter::new(output_sender);
    let mut zip = ZipFileWriter::new(writer);

    for path in body.into_inner() {
        if let Err(_) = storage_instance.send_file_stream(&input_sender, path).await {
            return HttpResponse::InternalServerError().body("Error sending file stream");
        }
    }

    tokio::spawn(async move {
        while let Some(file_stream) = input_receiver.recv().await {
            let mut stream = file_stream.stream;
            let entry =
                ZipEntryBuilder::new(ZipString::from(file_stream.name), Compression::Deflate);
            let mut entry_writer = match zip.write_entry_stream(entry).await {
                Ok(writer) => writer,
                Err(_) => return,
            };
            while let Some(Ok(chunk)) = stream.next().await {
                if let Err(_) = entry_writer.write_all(&chunk).await {
                    return;
                }
            }
            entry_writer.close().await.unwrap();
        }

        zip.close().await.unwrap();
    });

    HttpResponse::Ok()
        .content_type("application/zip")
        .insert_header(ContentDisposition::attachment("download.zip"))
        .streaming(response_stream)
}
