use crate::storage::FileStream;
use actix_web::Error;
use futures::{
    stream::Stream,
    task::{Context, Poll},
    StreamExt,
};
use std::io::Cursor;
use std::io::Write;
use std::pin::Pin;
use tokio::sync::mpsc;
use zip::{write::FileOptions, ZipWriter};

// Assume FileStream is defined elsewhere

pub struct ZipStream {
    zip_writer: Option<ZipWriter<Cursor<Vec<u8>>>>,
    receiver: mpsc::Receiver<FileStream>,
    buffer: Option<bytes::Bytes>, // Buffer to store the final ZIP data
}

impl ZipStream {
    pub fn new(receiver: mpsc::Receiver<FileStream>) -> ZipStream {
        println!("new zip");
        ZipStream {
            zip_writer: Some(ZipWriter::new(Cursor::new(Vec::new()))),
            receiver,
            buffer: None,
        }
    }

    // Asynchronous function to process files and create ZIP
    pub async fn process_files(&mut self) {
        while let Some(file_stream) = self.receiver.recv().await {
            println!("Processing file: {:?}", file_stream.name);
            if let Some(zip_writer) = self.zip_writer.as_mut() {
                zip_writer
                    .start_file(&file_stream.name, FileOptions::default())
                    .unwrap();
                let mut stream = file_stream.stream;

                while let Some(Ok(chunk)) = stream.next().await {
                    zip_writer.write_all(&chunk).unwrap();
                }
            }
        }

        if let Some(_zip_writer) = self.zip_writer.take() {
            let mut zip_writer = self.zip_writer.take().expect("Zip writer not found");
            let cursor = zip_writer.finish().expect("Failed to finish ZIP file");
            let data = cursor.into_inner();
            self.buffer = Some(bytes::Bytes::from(data));
        }
    }
}

impl Stream for ZipStream {
    type Item = Result<bytes::Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(buffer) = self.buffer.take() {
            Poll::Ready(Some(Ok(buffer)))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
