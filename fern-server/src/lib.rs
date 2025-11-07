//! Fern Server - A weird distributed WASM runtime 🌿
//!
//! This crate provides the core server functionality for the Fern distributed WASM runtime.
//! It includes modules for managing guest instances, data persistence, and server operations.

use std::path::PathBuf;

use anyhow::Result;
use iroh::{Endpoint, SecretKey, discovery::dns::DnsDiscovery, protocol::Router};
use log::info;
use tokio::{fs::File, io::AsyncWriteExt, task::LocalSet};

// Re-export core modules
pub mod data;
pub mod guest_instance;
pub mod server;
pub mod api;
pub mod cli;

// Re-export commonly used types
pub use data::Data;
pub use guest_instance::GuestInstance;
pub use server::{Server, GuestInfo};
pub use api::FernApiClient;

use crate::api::api_server;


/// Start a Fern server with the given secret key
pub async fn start_server(secret_path: Option<PathBuf>) -> Result<()> {
    let secret = if let Some(secret_path) = secret_path {
        load_secret_key(secret_path).await?
    } else {
        log::warn!("Key path not provided. Generating a random secret key");
        SecretKey::generate(&mut rand::rng())
    };

    let endpoint = Endpoint::builder()
        .discovery(DnsDiscovery::n0_dns())
        .secret_key(secret)
        .bind()
        .await?;

    let router_builder = Router::builder(endpoint.clone());

    log::info!("Starting Fern server {:#?}", endpoint.addr());

    // Create a LocalSet to run spawn_local tasks
    let local = LocalSet::new();

    local
        .run_until(async move {
            let server = Server::new(endpoint, router_builder);

            tokio::spawn(api_server(server));
            // let hello_world_module = include_bytes!("../../sample-guests/hello-world.wasm");

            // let r = server
            //     .create_module("hello-world".into(), hello_world_module.to_vec())
            //     .await;
            // info!("module create response {r:?}");
            // Keep the server running indefinitely

            //let mut n = 0;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                // n += 1;

                // if n % 10 == 0 {
                //     let r = server
                //         .update_module("hello-world".into(), hello_world_module.to_vec())
                //         .await;
                //     info!("module update response {r:?}");
                // }
            }
        })
        .await;

    Ok(())
}

/// Generate a new secret key and save it to the specified path
pub async fn generate_secret_key(path: PathBuf) -> Result<()> {
    let secret = SecretKey::generate(&mut rand::rng());

    let mut file = File::create(&path).await?;

    let secret_bytes = secret.to_bytes();

    file.write_all(&secret_bytes).await?;

    // Get the full absolute path
    let full_path = std::fs::canonicalize(&path)?;
    log::info!("secret saved to {}", full_path.display());
    Ok(())
}

/// Load a secret key from the specified file path
pub async fn load_secret_key(path: PathBuf) -> Result<SecretKey> {
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
