use super::Storage;
use crate::config;
use crate::storage::FileStream;
use crate::storage::ZipFile;
use async_trait::async_trait;
use tokio::sync::mpsc;

pub struct FtpStorage {
    pub config: config::StorageConfig,
}

#[async_trait]
impl Storage for FtpStorage {
    async fn connect(&self) -> bool {
        true
    }

    async fn get_file(&self, path: String) -> ZipFile {
        print!("get_file: {:?}", path);
        ZipFile {
            name: String::from(""),
            content: actix_web::web::Bytes::from(""),
        }
    }

    async fn send_file_stream(
        &self,
        _sender: &mpsc::Sender<FileStream>,
        _path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
