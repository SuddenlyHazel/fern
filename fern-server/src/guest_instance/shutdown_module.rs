use tokio::sync::oneshot;

pub struct ShutdownModule {
    pub reply: oneshot::Sender<ShutdownModuleResponse>,
}

pub struct ShutdownModuleResponse {
    pub success: bool,
    pub error_message: Option<String>,
}

pub(crate) async fn handle_shutdown_module(
    cmd: ShutdownModule,
    guest: &mut fern_runtime::guest::Guest,
) -> anyhow::Result<()> {
    let response = match perform_module_shutdown(guest).await {
        Ok(()) => ShutdownModuleResponse {
            success: true,
            error_message: None,
        },
        Err(e) => {
            log::error!("Failed to shutdown guest module: {}", e);
            ShutdownModuleResponse {
                success: false,
                error_message: Some(e.to_string()),
            }
        }
    };

    // Send response back
    if let Err(_) = cmd.reply.send(response) {
        log::warn!("Failed to send ShutdownModule response");
    }

    Ok(())
}

async fn perform_module_shutdown(guest: &mut fern_runtime::guest::Guest) -> anyhow::Result<()> {
    log::info!("Shutting down guest instance");
    
    // Gracefully shutdown the guest
    let _ = guest.shutdown();
    guest.endpoint.close().await;
    let _ = guest.router.shutdown().await;
    
    log::info!("Guest instance shutdown completed");
    Ok(())
}