pub mod config;
pub mod storage;
pub mod zip_stream;

use actix_web::{post, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use async_zip::base::write::ZipFileWriter;
use async_zip::Compression;
use async_zip::ZipEntryBuilder;
use async_zip::ZipString;
use bytes::Bytes;
use dotenv::dotenv;
use futures::stream::StreamExt;
use futures::SinkExt; // Required for try_send
use futures::{io::AsyncWrite, io::AsyncWriteExt};
use std::io::ErrorKind;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
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
    let (i_sender, mut i_receiver) = mpsc::channel(100);
    let (o_sender, o_receiver) = mpsc::channel::<bytes::Bytes>(10000);

    let response_stream = ReceiverStream::new(o_receiver).map(Ok::<_, Error>);
    let writer = CustomWriter::new(o_sender);
    let mut zip = ZipFileWriter::new(writer);

    for path in body.into_inner() {
        let _is = storage_instance.send_file_stream(&i_sender, path).await;
    }

    tokio::spawn(async move {
        while let Some(file_stream) = i_receiver.recv().await {
            println!("Sending {:?} to zip", file_stream.name);
            let mut stream = file_stream.stream;
            let entry =
                ZipEntryBuilder::new(ZipString::from(file_stream.name), Compression::Deflate);
            let mut entry_writer = zip.write_entry_stream(entry).await.unwrap();
            while let Some(Ok(chunk)) = stream.next().await {
                entry_writer.write_all(&chunk).await.unwrap();
            }
            entry_writer.close().await.unwrap();
        }

        zip.close().await.unwrap();
    });

    println!("Stream opened");

    HttpResponse::Ok()
        .content_type("application/zip")
        .append_header((
            "ContentDisposition",
            "attachment; filename=\"download.zip\"",
        ))
        .streaming(response_stream)
}
