use std::collections::btree_map::Entry;

use anyhow::anyhow;
use iroh::EndpointId;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::{
    data::{Data, GuestRow},
    guest_instance::UpdateModuleResponse,
    server::{InstanceMap, Server},
};

pub struct UpdateModule {
    pub name: String,
    pub module: Vec<u8>,
    pub reply: oneshot::Sender<UpdateResponse>,
}

impl Server {
    pub async fn update_module(
        &self,
        name: String,
        module: Vec<u8>,
    ) -> anyhow::Result<UpdateResponse> {
        let (tx, rx) = oneshot::channel();
        let cmd = UpdateModule {
            name,
            module,
            reply: tx,
        };

        self.sender.send(super::Commands::UpdateModule(cmd)).await?;

        Ok(rx.await?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResponse {
    pub success: bool,
    pub module_hash: String,
    pub previous_hash: Option<String>,
}

pub(crate) async fn handle_update_module(
    data: &Data,
    cmd: UpdateModule,
    instance_map: &mut InstanceMap,
    bootstrap: Vec<EndpointId>,
    guest_db_path: Option<std::path::PathBuf>,
) -> anyhow::Result<()> {
    let mut entry = match instance_map.entry(cmd.name.clone()) {
        Entry::Vacant(_) => {
            return Err(anyhow!("Guest with name {} does not exist", cmd.name));
        }
        Entry::Occupied(occupied_entry) => occupied_entry,
    };

    // Get the current guest to capture the previous hash
    let previous_hash = if let Some(current_guest) = GuestRow::by_name(data, &cmd.name)? {
        Some(current_guest.module_hash)
    } else {
        None
    };

    // Update the module (this will automatically save the old version to history)
    let db_update_success = GuestRow::update_module_by_name(data, &cmd.name, &cmd.module)?;

    // Calculate the new hash for the response
    let module_hash = blake3::hash(&cmd.module).to_string();

    // Only proceed with guest instance update if database update was successful
    let final_success = if db_update_success {
        // Send the command to update the guest instance
        let guest_instance = entry.get_mut();
        let UpdateModuleResponse {
            success: instance_update_success,
            error_message,
        } = guest_instance
            .update_module(cmd.module, module_hash.clone(), cmd.name.clone(), guest_db_path, bootstrap)
            .await?;

        if !instance_update_success {
            if let Some(error) = error_message {
                log::warn!("Guest instance update failed: {}", error);
            }
        }

        instance_update_success
    } else {
        log::warn!("Database update failed for guest: {}", cmd.name);
        false
    };

    cmd.reply
        .send(UpdateResponse {
            success: final_success,
            module_hash,
            previous_hash,
        })
        .map_err(|_| anyhow::anyhow!("Failed to send response"))?;

    Ok(())
}
