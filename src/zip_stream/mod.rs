use actix_web::Error;
use futures::{
    stream::Stream,
    task::{Context, Poll},
};
use std::pin::Pin;

pub struct ZipStream {
    buffer: Option<bytes::Bytes>,
}

impl ZipStream {
    pub fn new() -> ZipStream {
        println!("new zip stream");
        ZipStream { buffer: None }
    }

    pub fn append_buffer(&mut self, buffer: bytes::Bytes) {
        println!("append_buffer");
        self.buffer = Some(buffer);
    }
}

impl Stream for ZipStream {
    type Item = Result<bytes::Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        println!("poll_next");
        if let Some(buffer) = self.buffer.take() {
            println!("Buffer len: {:?}", buffer.len());
            Poll::Ready(Some(Ok(buffer)))
        } else {
            println!("Empty Buffer");
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
