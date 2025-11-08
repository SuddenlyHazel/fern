use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use fern_runtime::guest::Guest;
use iroh::EndpointId;
use log::warn;
use tokio::{
    sync::mpsc,
    time::{Duration, interval},
};

pub mod update_module;
pub use update_module::*;

pub mod shutdown_module;
pub use shutdown_module::*;

use crate::data::GuestRow;

pub enum GuestCommand {
    UpdateModule(update_module::UpdateModule),
    ShutdownModule(shutdown_module::ShutdownModule),
}

pub type CommandSender = mpsc::Sender<GuestCommand>;
pub type CommandReceiver = mpsc::Receiver<GuestCommand>;

pub struct GuestInstance {
    sender: CommandSender,
    node_id: EndpointId,
    pub module_hash: String,
    pub id : i64,
    handle: Arc<JoinHandle<anyhow::Result<()>>>,
}

impl GuestInstance {
    pub fn new(guest: Guest, module_hash: String, id : i64) -> Self {
        let (sender, receiver) = mpsc::channel(100);

        let node_id = guest.get_node_id();

        let handle = thread::spawn(move || guest_instance_thread(guest, receiver)).into();

        Self {
            handle,
            sender,
            node_id,
            id,
            module_hash
        }
    }

    pub async fn update_module(
        &mut self,
        module: Vec<u8>,
        module_hash: String,
        bootstrap: Vec<EndpointId>,
    ) -> anyhow::Result<update_module::UpdateModuleResponse> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let cmd = update_module::UpdateModule {
            module,
            module_hash: module_hash.clone(),
            bootstrap,
            reply: tx,
        };

        self.sender.send(GuestCommand::UpdateModule(cmd)).await?;
        let res = rx.await?;
        if res.success {
            self.module_hash = module_hash.clone();
        }
        Ok(res)
    }

    pub fn node_id(&self) -> EndpointId {
        self.node_id.clone()
    }

    /// Shutdown the guest instance gracefully
    pub async fn shutdown(&self) -> anyhow::Result<shutdown_module::ShutdownModuleResponse> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let cmd = shutdown_module::ShutdownModule { reply: tx };

        self.sender.send(GuestCommand::ShutdownModule(cmd)).await?;
        Ok(rx.await?)
    }
}

fn guest_instance_thread(guest: Guest, receiver: CommandReceiver) -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(guest_instance_task(receiver, guest))?;
    Ok(())
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
                if handle_command(cmd, &mut guest).await {
                    // If handle_command returns true, it means shutdown was requested
                    break;
                }
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
    guest.tick()?;
    Ok(())
}

async fn handle_command(cmd: GuestCommand, guest: &mut Guest) -> bool {
    match cmd {
        GuestCommand::UpdateModule(update_cmd) => {
            if let Err(e) = update_module::handle_update_module(update_cmd, guest).await {
                warn!("Failed to handle UpdateModule command: {}", e);
            }
            false // Continue running
        }
        GuestCommand::ShutdownModule(shutdown_cmd) => {
            if let Err(e) = shutdown_module::handle_shutdown_module(shutdown_cmd, guest).await {
                warn!("Failed to handle ShutdownModule command: {}", e);
            }
            true // Signal to exit the loop after shutdown
        }
    }
}
