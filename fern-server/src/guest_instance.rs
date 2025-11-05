use std::sync::Arc;

use fern_runtime::guest::Guest;
use iroh::EndpointId;
use log::warn;
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    time::{Duration, interval},
};

pub mod update_module;
pub use update_module::*;

pub enum GuestCommand {
    UpdateModule(update_module::UpdateModule),
}

pub type CommandSender = mpsc::Sender<GuestCommand>;
pub type CommandReceiver = mpsc::Receiver<GuestCommand>;

pub struct GuestInstance {
    sender: CommandSender,
    node_id: EndpointId,
    handle: Arc<JoinHandle<anyhow::Result<()>>>,
}

impl GuestInstance {
    pub fn new(guest: Guest) -> Self {
        let (sender, receiver) = mpsc::channel(100);

        let node_id = guest.get_node_id();
        let handle = tokio::spawn(guest_instance_task(receiver, guest)).into();
        Self {
            handle,
            sender,
            node_id,
        }
    }

    pub async fn update_module(
        &self,
        module: Vec<u8>,
        module_hash: String,
        bootstrap: Vec<EndpointId>,
    ) -> anyhow::Result<update_module::UpdateModuleResponse> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let cmd = update_module::UpdateModule {
            module,
            module_hash,
            bootstrap,
            reply: tx,
        };

        self.sender.send(GuestCommand::UpdateModule(cmd)).await?;
        Ok(rx.await?)
    }

    pub fn node_id(&self) -> EndpointId {
        self.node_id.clone()
    }
}

async fn guest_instance_task(
    mut receiver: CommandReceiver,
    mut guest: Guest,
) -> anyhow::Result<()> {
    // Create interval for 5 times per second (200ms)
    let mut tick_interval = interval(Duration::from_millis(200));

    loop {
        tokio::select! {
            // Handle guest ticking at 5Hz
            _ = tick_interval.tick() => {
                if let Err(e) = tick_guest(&mut guest).await {
                    warn!("failed to tick guest {e}");
                }
            }

            // Handle incoming commands
            Some(cmd) = receiver.recv() => {
                handle_command(cmd, &mut guest).await;
            }

            // If the receiver is closed, exit the loop
            else => {
                break;
            }
        }
    }
    Ok(())
}

async fn tick_guest(guest: &mut Guest) -> anyhow::Result<()> {
    guest.tick_gossip().await?;
    Ok(())
}

async fn handle_command(cmd: GuestCommand, guest: &mut Guest) {
    match cmd {
        GuestCommand::UpdateModule(update_cmd) => {
            if let Err(e) = update_module::handle_update_module(update_cmd, guest).await {
                warn!("Failed to handle UpdateModule command: {}", e);
            }
        }
    }
}
