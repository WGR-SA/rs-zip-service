use crate::config::StorageConfig;
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::Stream;

use std::pin::Pin;
use tokio::sync::mpsc;

pub mod ftp;
pub mod http;
pub mod openstack;
pub mod s3;

#[derive(Debug)]
pub struct ZipFile {
    pub name: String,
    pub content: Bytes,
}

pub struct FileStream {
    pub name: String,
    pub stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, std::io::Error>> + Send>>,
}

#[async_trait]
pub trait Storage {
    async fn connect(&self) -> bool;
    async fn get_file(&self, path: String) -> ZipFile;
    async fn send_file_stream(
        &self,
        sender: &mpsc::Sender<FileStream>,
        path: String,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub fn get_storage(config: StorageConfig) -> Box<dyn Storage> {
    match config.provider.as_str() {
        "openstack" => Box::new(openstack::OpenstackStorage { config }),
        "s3" => Box::new(s3::S3Storage { config }),
        "ftp" => Box::new(ftp::FtpStorage { config }),
        _ => Box::new(http::HttpStorage { config }),
    }
}
