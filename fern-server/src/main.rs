use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use iroh::{Endpoint, SecretKey, discovery::dns::DnsDiscovery, protocol::Router};
use log::{error, info, warn};
use tokio::{fs::File, io::AsyncWriteExt};

pub mod data;
pub mod guest_instance;
pub mod server;

/// Fern Server - A weird distributed WASM runtime 🌿
#[derive(Parser)]
#[command(name = "fern-server")]
#[command(about = "A weird distributed WASM runtime 🌿")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the Fern server
    Start {
        /// Path to identity secret will generate random identity if empty
        #[arg(long)]
        secret: Option<PathBuf>,
    },
    /// Generate a new identity secret key for the Fern server
    GenerateSecret {
        /// Path where the secret key file will be saved (e.g., ./fern-secret.key)
        #[arg(
            long,
            help = "Output path for the generated secret key file (e.g., ./fern-secret.key)"
        )]
        path: PathBuf,
    },
}

async fn handle_start_command(secret_path: Option<PathBuf>) -> Result<()> {
    let secret = if let Some(secret_path) = secret_path {
        load_secret_key(secret_path).await?
    } else {
        warn!("Key path not provided. Generating a random secret key");
        SecretKey::generate(&mut rand::rng())
    };

    let endpoint = Endpoint::builder()
        .discovery(DnsDiscovery::n0_dns())
        .secret_key(secret)
        .bind()
        .await?;

    let router_builder = Router::builder(endpoint.clone());

    info!("Starting Fern server {:#?}", endpoint.addr());

    Ok(())
}

async fn handle_generate_secret_command(path: PathBuf) -> Result<()> {
    let secret = SecretKey::generate(&mut rand::rng());

    let mut file = File::create(&path).await?;

    let secret_bytes = secret.to_bytes();

    file.write_all(&secret_bytes).await?;

    // Get the full absolute path
    let full_path = std::fs::canonicalize(&path)?;
    info!("secret saved to {}", full_path.display());
    Ok(())
}

/// Load a secret key from the specified file path
async fn load_secret_key(path: PathBuf) -> Result<SecretKey> {
    use tokio::io::AsyncReadExt;

    let mut file = File::open(&path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    // SecretKey expects exactly 32 bytes
    if buffer.len() != 32 {
        return Err(anyhow::anyhow!(
            "Invalid secret key file: expected 32 bytes, got {} bytes",
            buffer.len()
        ));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&buffer);

    Ok(SecretKey::from(key_bytes))
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Start { secret } => handle_start_command(secret).await,
        Commands::GenerateSecret { path } => handle_generate_secret_command(path).await,
    };

    if let Err(e) = result {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}
