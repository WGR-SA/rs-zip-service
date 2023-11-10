use std::vec;

use super::Storage;
use crate::config;

pub struct OpenstackStorage {
    pub config: config::StorageConfig,
}

impl Storage for OpenstackStorage {
    fn connect(&self) -> bool {
        true
    }

    fn get_files(&self, files: Vec<String>) -> Vec<u8> {
        // create base url with url & user string
        let base_url = format!("{}{}", self.config.url, self.config.user);

        for file in files {}

        vec![]
    }
}
