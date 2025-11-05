use fern_runtime::{guest::new_guest, iroh_helpers::iroh_bundle};
use iroh::{Endpoint, EndpointId, protocol::{Router, RouterBuilder}};
use iroh_gossip::Gossip;
use tokio::sync::{mpsc, oneshot};

use crate::{data::Data, server::gossip::setup_gossip};

pub mod create_module;
pub use create_module::*;

pub mod update_bootstrap;
pub use update_bootstrap::*;

pub mod gossip;

pub enum Commands {
    CreateModule(CreateModule),
    UpdateBootstrap(UpdateBootstrap),
}

pub type CommandReceiver = mpsc::Receiver<Commands>;
pub type CommandSender = mpsc::Sender<Commands>;

#[derive(Clone)]
pub struct Server {
    sender: CommandSender,
}

impl Server {
    pub fn new(endpoint: Endpoint, router_builder: RouterBuilder) -> Self {
        let (sender, rx) = mpsc::channel(100);

        tokio::task::spawn_local(server_task(endpoint, router_builder, rx));

        Self { sender }
    }
}

pub async fn server_task(
    endpoint: Endpoint,
    router_builder: RouterBuilder,
    mut command_receiver: CommandReceiver,
) -> anyhow::Result<()> {
    let data = Data::new_memory();

    let (router_builder, gossip) = setup_gossip(router_builder, endpoint.clone());

    let router = router_builder.spawn();

    // TODO we should store additional known peers somewhere..
    let mut bootstrap = vec![endpoint.id()];

    while let Some(cmd) = command_receiver.recv().await {
        let res = match cmd {
            Commands::CreateModule(create_module) => {
                handle_create_module(&data, create_module, bootstrap.clone()).await
            }
            Commands::UpdateBootstrap(update_bootstrap) => {
                handle_update_bootstrap(update_bootstrap, &mut bootstrap).await
            }
        };
    }

    Ok(())
}
