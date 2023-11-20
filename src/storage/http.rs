use super::Storage;
use crate::config;
use crate::storage::FileStream;
use crate::storage::ZipFile;
use async_trait::async_trait;
use futures::TryStreamExt;
use reqwest;
use tokio::sync::mpsc;

pub struct HttpStorage {
    pub config: config::StorageConfig,
}

#[async_trait]
impl Storage for HttpStorage {
    async fn connect(&self) -> bool {
        println!("connect: to openstack");
        true
    }

    async fn get_file(&self, path: String) -> ZipFile {
        print!("get_file: {:?}", path);
        let response = reqwest::get(&path).await.unwrap();

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
        let response = reqwest::get(&path).await?;

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
