use bytes::Bytes;
use futures::io::AsyncWrite;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

pub struct ZipWriter {
    sender: mpsc::Sender<bytes::Bytes>,
}

impl ZipWriter {
    // Fields to hold state and possibly communicate with the response stream
    pub fn new(sender: mpsc::Sender<bytes::Bytes>) -> ZipWriter {
        ZipWriter { sender: sender }
    }
}

impl AsyncWrite for ZipWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let data: Bytes = Bytes::copy_from_slice(&buf);
        match self.sender.try_send(data) {
            Ok(_) => Poll::Ready(Ok(buf.len())),
            Err(_) => {
                println!("Retry");
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}
