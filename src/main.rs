pub mod config;
pub mod storage;
pub mod zip_stream;

use crate::zip_stream::ZipStream;
use actix_web::{post, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use async_zip::base::write::ZipFileWriter;
use async_zip::Compression;
use async_zip::ZipEntryBuilder;
use async_zip::ZipString;
use dotenv::dotenv;
use futures::stream::StreamExt;
use futures::{io::AsyncWrite, io::AsyncWriteExt, stream::TryStreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
// use zip_stream::ZipStream;

pub struct CustomWriter {
    sender: mpsc::Sender<bytes::Bytes>,
}

impl CustomWriter {
    // Fields to hold state and possibly communicate with the response stream
    pub fn new(o_sender: mpsc::Sender<bytes::Bytes>) -> CustomWriter {
        CustomWriter { sender: o_sender }
    }

    pub fn write_all_buf(&mut self, buf: &mut bytes::Bytes) {
        self.sender.try_send(buf.clone()).unwrap();
    }
}

impl AsyncWrite for CustomWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        println!("poll_write {:?}", buf.len());
        //self.sender.try_send(bytes::Bytes::from(buf)).unwrap();
        Poll::Ready(Ok(buf.len()))
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
    let (i_sender, mut i_receiver) = mpsc::channel(32);
    let (o_sender, mut o_receiver) = mpsc::channel(32);

    let mut writer = CustomWriter::new(o_sender);
    let mut zip = ZipFileWriter::new(&mut writer);

    for path in body.into_inner() {
        let _is = storage_instance.send_file_stream(&i_sender, path).await;
    }

    while let Some(file_stream) = i_receiver.recv().await {
        println!("Sending {:?} to zip", file_stream.name);
        let mut stream = file_stream.stream;
        let entry = ZipEntryBuilder::new(ZipString::from(file_stream.name), Compression::Deflate);
        let mut entry_writer = zip.write_entry_stream(entry).await.unwrap();
        while let Some(Ok(chunk)) = stream.next().await {
            entry_writer.write_all(&chunk).await.unwrap();
        }
        entry_writer.close().await.unwrap();
        println!("Sended");
    }

    writer.close().await.unwrap();
    println!("Finished sending files to zip stream");

    //stream.process_files();
    //let response_stream = o_receiver.map(Ok::<_, Error>);
    let response_stream = ZipStream::new();

    HttpResponse::Ok()
        .content_type("application/zip")
        .append_header((
            "ContentDisposition",
            "attachment; filename=\"download.zip\"",
        ))
        .streaming(response_stream)
}
