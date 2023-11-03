use crate::config;
use crate::storage;

pub fn handle_compress_files(body: Vec<String>, client: String) -> String {
    println!("handler");

    let config: config::StorageConfig = config::get_config(client).unwrap();
    println!("config: {:?}", config);

    let storage = storage::get_storage(config);

    // let files = storage::get_files(body);

    format!("{:?}", body)
}
