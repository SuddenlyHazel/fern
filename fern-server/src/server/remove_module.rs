use std::collections::btree_map::Entry;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::{
    data::{Data, GuestRow},
    server::{InstanceMap, Server},
};

pub struct RemoveModule {
    pub name: String,
    pub reply: oneshot::Sender<RemoveResponse>,
}

impl Server {
    pub async fn remove_module(&self, name: String) -> anyhow::Result<RemoveResponse> {
        let (tx, rx) = oneshot::channel();
        let cmd = RemoveModule { name, reply: tx };

        self.sender.send(super::Commands::RemoveModule(cmd)).await?;

        Ok(rx.await?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveResponse {
    pub success: bool,
    pub message: String,
}

pub(crate) async fn handle_remove_module(
    data: &Data,
    cmd: RemoveModule,
    instance_map: &mut InstanceMap,
) -> anyhow::Result<()> {
    let entry = match instance_map.entry(cmd.name.clone()) {
        Entry::Vacant(_) => {
            let response = RemoveResponse {
                success: false,
                message: format!("Guest with name '{}' does not exist", cmd.name),
            };
            
            cmd.reply
                .send(response)
                .map_err(|_| anyhow::anyhow!("Failed to send response"))?;
            return Ok(());
        }
        Entry::Occupied(occupied_entry) => occupied_entry,
    };

    // 1. Gracefully shutdown the guest instance first
    log::info!("Shutting down guest instance: {}", cmd.name);
    let guest_instance = entry.get();
    
    let shutdown_result = guest_instance.shutdown().await;
    
    // 2. Remove from the instance map
    entry.remove();
    log::info!("Removed guest instance from memory: {}", cmd.name);

    // 3. Remove from the database
    let db_removal_success = GuestRow::remove_by_name(data, &cmd.name)?;
    
    let response = match (shutdown_result, db_removal_success) {
        (Ok(shutdown_response), true) if shutdown_response.success => {
            RemoveResponse {
                success: true,
                message: format!("Successfully removed guest '{}'", cmd.name),
            }
        }
        (Ok(shutdown_response), true) => {
            RemoveResponse {
                success: false,
                message: format!("Guest '{}' removed from database but shutdown failed: {}",
                    cmd.name,
                    shutdown_response.error_message.unwrap_or_else(|| "Unknown error".to_string())
                ),
            }
        }
        (_, false) => {
            RemoveResponse {
                success: false,
                message: format!("Failed to remove guest '{}' from database", cmd.name),
            }
        }
        (Err(e), _) => {
            RemoveResponse {
                success: false,
                message: format!("Failed to shutdown guest '{}': {}", cmd.name, e),
            }
        }
    };

    cmd.reply
        .send(response)
        .map_err(|_| anyhow::anyhow!("Failed to send response"))?;

    log::info!("Guest removal completed for: {}", cmd.name);
    Ok(())
}