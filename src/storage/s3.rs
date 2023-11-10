use super::Storage;
use crate::config;

pub struct S3Storage {
    pub config: config::StorageConfig,
}

impl Storage for S3Storage {
    fn connect(&self) -> bool {
        true
    }

    fn get_files(&self, files: Vec<String>) -> Vec<u8> {
        print!("get_files: {:?}", files);
        vec![]
    }
}
