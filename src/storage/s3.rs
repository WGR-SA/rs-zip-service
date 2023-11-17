use super::Storage;
use crate::config;
use crate::storage::FileStream;
use crate::storage::ZipFile;
use async_trait::async_trait;
use tokio::sync::mpsc;

pub struct S3Storage {
    pub config: config::StorageConfig,
}

#[async_trait]
impl Storage for S3Storage {
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
        sender: &mpsc::Sender<FileStream>,
        path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // trace!("bucket:      {}", opt.bucket);
        // trace!("object:      {}", opt.object);
        // trace!("destination: {}", opt.destination.display());

        // let mut file = File::create(opt.destination.clone())?;

        // let mut object = client
        //     .get_object()
        //     .bucket(opt.bucket)
        //     .key(opt.object)
        //     .send()
        //     .await?;

        // let mut byte_count = 0_usize;
        // while let Some(bytes) = object.body.try_next().await? {
        //     let bytes_len = bytes.len();
        //     file.write_all(&bytes)?;
        //     trace!("Intermediate write of {bytes_len}");
        //     byte_count += bytes_len;
        // }

        Ok(())
    }
}
