use std::path::PathBuf;

use iroh::PublicKey;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub runtime_data_dir: PathBuf,
    pub iroh_secret_path: PathBuf,
    pub bootstrap_peers: Vec<PublicKey>,
}

impl Config {
    pub async fn try_from_path(config_path: PathBuf) -> Self {
        let mut file = File::open(config_path).await.expect("failed to open file");
        let mut config_str = String::new();
        file.read_to_string(&mut config_str)
            .await
            .expect("failed to read file in");
        toml::from_str(&config_str).expect("failed to parse config file")
    }
}
