pub mod config;
pub mod storage;
pub mod zip_stream;

use actix_cors::Cors;
use actix_web::{
    http::header::ContentDisposition, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use async_zip::base::write::ZipFileWriter;
use async_zip::Compression;
use async_zip::ZipEntryBuilder;
use async_zip::ZipString;
use bytes::Bytes;
use dotenv::dotenv;
use futures::stream::StreamExt;
use futures::{io::AsyncWrite, io::AsyncWriteExt};
use std::env;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct CustomWriter {
    sender: mpsc::Sender<bytes::Bytes>,
}

impl CustomWriter {
    // Fields to hold state and possibly communicate with the response stream
    pub fn new(o_sender: mpsc::Sender<bytes::Bytes>) -> CustomWriter {
        CustomWriter { sender: o_sender }
    }
}

impl AsyncWrite for CustomWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let data: Bytes = Bytes::copy_from_slice(&buf);
        println!("Writing {:?} bytes", buf.len());
        match self.sender.try_send(data) {
            Ok(_) => Poll::Ready(Ok(buf.len())),
            Err(_) => {
                // The channel might be full or closed. In either case, register for wakeup
                // and return Pending.
                println!("Retry");
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load variables from `.env`
    println!("hello");

    HttpServer::new(|| {
        let cors = Cors::permissive();
        // .allowed_origin("*")
        // .allowed_methods(vec!["GET", "POST"])
        // .allowed_headers(vec![
        //     http::header::AUTHORIZATION,
        //     http::header::ACCEPT,
        //     http::header::CONTENT_TYPE,
        //     http::header::HeaderName::from_static("X-Client"),
        // ])
        // .max_age(3600);

        App::new().wrap(cors).service(get_files_stream_zip)
    })
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
    let (i_sender, mut i_receiver) = mpsc::channel(100);
    let (o_sender, o_receiver) = mpsc::channel::<bytes::Bytes>(10000);

    let response_stream = ReceiverStream::new(o_receiver).map(Ok::<_, Error>);
    let writer = CustomWriter::new(o_sender);
    let mut zip = ZipFileWriter::new(writer);

    for path in body.into_inner() {
        if let Err(_) = storage_instance.send_file_stream(&i_sender, path).await {
            return HttpResponse::InternalServerError().body("Error sending file stream");
        }
    }

    tokio::spawn(async move {
        while let Some(file_stream) = i_receiver.recv().await {
            println!("Sending {:?} to zip", file_stream.name);
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
    println!("Stream opened");

    HttpResponse::Ok()
        .content_type("application/zip")
        .insert_header(ContentDisposition::attachment("download.zip"))
        .streaming(response_stream)
}
