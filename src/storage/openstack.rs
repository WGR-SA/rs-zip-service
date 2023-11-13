use super::Storage;
use crate::config;
use crate::storage::FileStream;
use crate::storage::ZipFile;
use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest::{self, Error};
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio_util::codec::{BytesCodec, FramedRead}; // Import StreamExt

pub struct OpenstackStorage {
    pub config: config::StorageConfig,
}

#[async_trait]
impl Storage for OpenstackStorage {
    async fn connect(&self) -> bool {
        println!("connect: to openstack");
        true
    }

    async fn get_file(&self, path: String) -> ZipFile {
        println!("get file: from openstack");

        let base_url = format!("{}/{}", self.config.url, self.config.user);
        let file_url = format!("{}{}", base_url, path);
        println!("url: {:?}", file_url);

        let response = reqwest::get(file_url).await.unwrap();

        ZipFile {
            name: String::from(path.split("/").last().unwrap()),
            content: response.bytes().await.unwrap(),
        }
    }

    async fn send_file_stream(
        &self,
        sender: &mpsc::Sender<FileStream>,
        path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("send file stream: {}", path);
        let base_url = format!("{}/{}", self.config.url, self.config.user);
        let file_url = format!("{}{}", base_url, path);

        let response = reqwest::get(&file_url).await?;

        // Ensure we got a success status
        if !response.status().is_success() {
            return Err(("Unexpected status code").into());
        }

        let stream = response
            .bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

        let file_stream = FileStream {
            name: path.split('/').last().unwrap_or("unknown").to_string(),
            stream: Box::pin(stream),
        };

        sender.send(file_stream).await?;
        Ok(())
    }
}
