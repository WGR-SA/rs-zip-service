use super::Storage;
use crate::config;
use crate::storage::FileStream;
use crate::storage::ZipFile;
use async_stream::stream;
use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use tokio::sync::mpsc;

pub struct S3Storage {
    pub config: config::StorageConfig,
}

#[async_trait]
impl Storage for S3Storage {
    async fn connect(&self) -> bool {
        std::env::set_var("AWS_ACCESS_KEY_ID", &self.config.user);
        std::env::set_var("AWS_SECRET_ACCESS_KEY", &self.config.secret);
        std::env::set_var("AWS_DEFAULT_REGION", &self.config.region);

        let region_provider = RegionProviderChain::default_provider().or_else("eu-central-1");
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let _client = Client::new(&aws_config);
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
        let region_provider = RegionProviderChain::default_provider().or_else("eu-central-1");
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;

        let client = Client::new(&aws_config);
        let filepath = format!("{}/{}", self.config.url, path);
        let byte_stream = client
            .get_object()
            .bucket(&self.config.bucket)
            .key(&filepath)
            .send()
            .await?
            .body;

        let stream = stream! {
            tokio::pin!(byte_stream);
            while let Some(next) = byte_stream.next().await {
                match next {
                    Ok(data) => yield Ok(data),
                    Err(e) => yield Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
                }
            }
        };

        let file_stream = FileStream {
            name: path,
            stream: Box::pin(stream),
        };

        sender.send(file_stream).await?;

        Ok(())
    }
}
