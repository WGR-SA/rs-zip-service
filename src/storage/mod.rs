use crate::config::StorageConfig;

pub mod local;
pub mod s3;
// pub mod openstack;

pub trait Storage {
    fn connect(&self) -> ();
    fn get_files(&self, files: Vec<String>) -> Vec<u8>;
}

pub fn get_storage(config: StorageConfig) -> Box<dyn Storage> {
    match config.provider.as_str() {
        "s3" => Box::new(s3::S3Storage { config }),
        _ => Box::new(local::LocalStorage { config }),
    }
}
