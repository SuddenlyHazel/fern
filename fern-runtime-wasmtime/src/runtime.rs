use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use iroh::{PublicKey, SecretKey};
use log::{info, warn};
use owo_colors::OwoColorize;
use rand::TryRngCore;

use crate::{iroh_helpers::iroh_bundle, runtime::fern_gossip::FernGossip, runtime_config::Config};
pub mod fern_gossip;
pub struct Runtime {}

static FERN_ASCII: &str = r#"

    ███████ ███████ ██████  ███    ██ 
    ██      ██      ██   ██ ████   ██ 
    █████   █████   ██████  ██ ██  ██ 
    ██      ██      ██   ██ ██  ██ ██ 
    ██      ███████ ██   ██ ██   ████                        

        -=[Whacky weird WASM]=-
"#;

pub async fn start(config: Option<Config>, peers_maybe: Vec<PublicKey>) -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .filter_module("iroh", log::LevelFilter::Error)
        .filter_module("tracing::span", log::LevelFilter::Error)
        .init();

    let config = config.unwrap_or_else(|| create_tmp_config(peers_maybe));

    let secret_key = iroh_secret_from_path(&config.iroh_secret_path)?;
    info!("Iroh Public Key {}", secret_key.public());

    let iroh_bundle = iroh_bundle(secret_key, config.bootstrap_peers.clone()).await?;

    let runtime_gossip = FernGossip::builder(iroh_bundle.gossip.clone())
        .add_peers(config.bootstrap_peers)
        .spawn()
        .await?;

    println!("{}", FERN_ASCII.on_purple());

    tokio::signal::ctrl_c().await;
    warn!("shutting down");
    Ok(())
}

fn iroh_secret_from_path(path: &Path) -> anyhow::Result<SecretKey> {
    let mut file = File::open(path)?;

    let mut bytes = [0; 32];
    file.read_exact(&mut bytes)?;

    Ok(SecretKey::from_bytes(&bytes))
}

fn create_tmp_config(peers_maybe: Vec<PublicKey>) -> Config {
    let secret = {
        let mut os_rng = rand::rngs::OsRng::default();
        let mut bytes = [0; 32];
        os_rng
            .try_fill_bytes(&mut bytes)
            .expect("failed to get entropy from OS Rng?");
        SecretKey::from_bytes(&bytes)
    };
    let data_path = tempfile::TempDir::new()
        .expect("failed to get temp dir path")
        .keep();

    warn!(
        "Config was not provided. Creating a temp data store at {}",
        data_path.display()
    );
    let secret_path = data_path.join("secret.key");
    let mut secret_key_file = File::create(&secret_path).expect("failed to create secret key file");
    secret_key_file
        .write_all(&secret.to_bytes())
        .expect("failed to write secret key to file");

    Config {
        runtime_data_dir: data_path,
        iroh_secret_path: secret_path,
        bootstrap_peers: peers_maybe,
    }
}
