use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{error, info};

use fern_server::{FernApiClient, cli::{GuestsTable, GuestsTableProps}, generate_secret_key, start_server};
use iocraft::prelude::*;

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
    HealthCheck {},
    ListGuests {},
    CreateModule {
        name : String,
        module_path: PathBuf
    }
}

async fn handle_start_command(secret_path: Option<PathBuf>) -> Result<()> {
    start_server(secret_path).await
}

async fn handle_generate_secret_command(path: PathBuf) -> Result<()> {
    generate_secret_key(path).await
}

async fn handle_health_check_command() -> Result<()> {
    let client = FernApiClient::localhost();
    if client.health_check().await {
        element! {
            View(
            border_style: BorderStyle::Round,
            border_color: Color::Blue,
        ) {
            Text(content: "ONLINE")
        }
        }
        .print();
    } else {
        element! {
            View(
            border_style: BorderStyle::Round,
            border_color: Color::Red,
        ) {
            Text(content: "OFFLINE")
        }
        }
        .print();
    }

    Ok(())
}

async fn handle_list_guest_command() -> Result<()> {
    let client = FernApiClient::localhost();
    let guests = client.list_guests().await?;

    // let guests = GuestsTableProps {
    //     guests: ,
    // };

    element! {
        GuestsTable(guests: Some(guests))
    }
    .print();

    Ok(())
}

async fn handle_create_module_command(name: String, module_path: PathBuf) -> Result<()> {
    let client = FernApiClient::localhost();
    
    // Read the module file
    let module_bytes = std::fs::read(&module_path)
        .map_err(|e| anyhow::anyhow!("Failed to read module file at {:?}: {}", module_path, e))?;
    
    // Create the guest module
    match client.create_guest(name.clone(), module_bytes).await {
        Ok(response) => {
            element! {
                View(
                    border_style: BorderStyle::Round,
                    border_color: Color::Green,
                    padding: 1,
                ) {
                    Text(content: format!("✅ Successfully created guest '{}'", name), weight: Weight::Bold)
                    Text(content: format!("Endpoint ID: {}", response.endpoint_id))
                }
            }
            .print();
        }
        Err(e) => {
            element! {
                View(
                    border_style: BorderStyle::Round,
                    border_color: Color::Red,
                    padding: 1,
                ) {
                    Text(content: format!("❌ Failed to create guest '{}'", name), weight: Weight::Bold)
                    Text(content: format!("Error: {}", e))
                }
            }
            .print();
            return Err(e);
        }
    }
    
    Ok(())
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
        Commands::HealthCheck {} => handle_health_check_command().await,
        Commands::ListGuests {} => handle_list_guest_command().await,
        Commands::CreateModule { name, module_path } => handle_create_module_command(name, module_path).await,
    };

    if let Err(e) = result {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}
