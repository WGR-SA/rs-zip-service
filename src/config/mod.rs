use std::env;

#[derive(Debug)]
pub struct StorageConfig {
    pub provider: String,
    pub url: String,
    pub user: String,
    pub secret: String,
    pub region: String,
    pub bucket: String,
}

impl StorageConfig {
    pub fn set(&mut self, key: String, value: String) {
        match key.as_str() {
            "provider" => self.provider = value,
            "url" => self.url = value,
            "user" => self.user = value,
            "secret" => self.secret = value,
            "region" => self.region = value,
            "bucket" => self.bucket = value,
            _ => println!("unknown key: {}", key),
        }
    }
}

fn default() -> StorageConfig {
    StorageConfig {
        provider: String::from("http"),
        url: String::from(""),
        user: String::from(""),
        secret: String::from(""),
        region: String::from(""),
        bucket: String::from(""),
    }
}

pub fn get_config(client: &str) -> Result<StorageConfig, std::io::Error> {
    let config = self::get_from_env(client).unwrap();
    Ok(config)
}

fn get_from_env(client: &str) -> Result<StorageConfig, std::io::Error> {
    let mut config = self::default();

    for (key, value) in env::vars() {
        let mut prefix = String::from(client);
        prefix.push_str("_STORAGE_");

        if key.starts_with(&prefix) {
            let storage_key = key.replace(&prefix, "").to_lowercase();
            config.set(storage_key, value);
        }
    }
    Ok(config)
}
