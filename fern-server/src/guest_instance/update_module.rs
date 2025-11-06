use std::mem;

use fern_runtime::{
    guest::{Guest, new_guest},
    iroh_helpers::iroh_bundle_with_secret,
};
use iroh::EndpointId;
use tokio::sync::oneshot;

pub struct UpdateModule {
    pub module: Vec<u8>,
    pub module_hash: String,
    pub reply: oneshot::Sender<UpdateModuleResponse>,
    pub bootstrap: Vec<EndpointId>,
}

pub struct UpdateModuleResponse {
    pub success: bool,
    pub error_message: Option<String>,
}

pub(crate) async fn handle_update_module(
    cmd: UpdateModule,
    guest: &mut fern_runtime::guest::Guest,
) -> anyhow::Result<()> {
    let response = match perform_module_update(cmd.module, cmd.bootstrap, guest).await {
        Ok(()) => UpdateModuleResponse {
            success: true,
            error_message: None,
        },
        Err(e) => {
            log::error!("Failed to update guest module: {}", e);
            UpdateModuleResponse {
                success: false,
                error_message: Some(e.to_string()),
            }
        }
    };

    // Send response back
    if let Err(_) = cmd.reply.send(response) {
        log::warn!("Failed to send UpdateModule response");
    }

    Ok(())
}

async fn perform_module_update(
    module: Vec<u8>,
    bootstrap: Vec<EndpointId>,
    guest: &mut Guest,
) -> anyhow::Result<()> {
    // 1. Capture the secret key to maintain network identity
    let secret_key = guest.endpoint.secret_key().clone();

    // 2. Gracefully shutdown the existing guest
    log::info!("Shutting down existing guest instance");
    let _ = guest.shutdown();
    guest.endpoint.close().await;
    let _ = guest.router.shutdown().await;

    // 3. Create a new guest with the updated module using the same secret key
    log::info!("Creating new guest instance with updated module");
    let (endpoint, router_builder) = iroh_bundle_with_secret(secret_key).await?;
    let mut new_guest = new_guest(module, (endpoint, router_builder, bootstrap))?;

    // TODO how should we handle a guest failing to initialize here?
    // at the very least we should report it..
    // Should we rollback? Just allow it to be created and failing?
    let _ = new_guest.initialize();

    // 4. Swap the guests - the old guest will be dropped
    // NOTE: After this swap, `new_guest` contains the old guest instance
    mem::swap(guest, &mut new_guest);

    // 5. Clean up the old guest instance (now in new_guest)
    //    we don't really have to force a drop. But, this way
    //    accidents might be avoided in future.
    drop(new_guest);

    log::info!("Guest module update completed successfully");
    Ok(())
}
