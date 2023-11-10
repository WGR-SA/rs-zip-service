use super::Storage;
use crate::config;

pub struct LocalStorage {
    pub config: config::StorageConfig,
}

impl Storage for LocalStorage {
    fn connect(&self) -> bool {
        true
    }

    fn get_files(&self, files: Vec<String>) -> Vec<u8> {
        print!("get_files: {:?}", files);
        vec![]
    }
}
