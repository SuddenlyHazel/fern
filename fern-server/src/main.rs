use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::error;

use fern_server::{start_server, generate_secret_key};

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
    start_server(secret_path).await
}

async fn handle_generate_secret_command(path: PathBuf) -> Result<()> {
    generate_secret_key(path).await
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
