use super::Storage;
use crate::config;
use crate::storage::FileStream;
use crate::storage::ZipFile;
use actix_web::web::Bytes;
use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use futures::{
    stream::Stream,
    task::{Context, Poll},
};
use std::pin::Pin;
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

        // let mut object = client
        //     .get_object()
        //     .bucket(&self.config.bucket)
        //     .key(path)
        //     .send()
        //     .await?;

        // let stream = object.body.take().unwrap();

        // while let Some(bytes) = object.body.try_next().await? {}

        // let file_stream = FileStream {
        //     name: path.split('/').last().unwrap_or("unknown").to_string(),
        //     stream: stream,
        // };

        // sender.send(file_stream).await?;

        Ok(())
    }
}
