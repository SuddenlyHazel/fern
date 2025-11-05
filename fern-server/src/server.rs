use std::{collections::BTreeMap, sync::Arc};

use iroh::{Endpoint, protocol::RouterBuilder};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{data::Data, guest_instance::GuestInstance, server::gossip::setup_gossip};

pub mod create_module;
pub use create_module::*;

pub mod update_bootstrap;
pub use update_bootstrap::*;

pub mod update_module;
pub use update_module::*;

pub mod gossip;

pub enum Commands {
    CreateModule(CreateModule),
    UpdateBootstrap(UpdateBootstrap),
    UpdateModule(UpdateModule),
}

pub type CommandReceiver = mpsc::Receiver<Commands>;
pub type CommandSender = mpsc::Sender<Commands>;
pub type InstanceMap = BTreeMap<String, GuestInstance>;

#[derive(Clone)]
pub struct Server {
    sender: CommandSender,
    task: Arc<JoinHandle<anyhow::Result<()>>>,
}

impl Server {
    pub fn new(endpoint: Endpoint, router_builder: RouterBuilder) -> Self {
        let (sender, rx) = mpsc::channel(100);

        let task = Arc::new(tokio::task::spawn_local(server_task(
            endpoint,
            router_builder,
            rx,
        )));

        Self { sender, task }
    }
}

pub async fn server_task(
    endpoint: Endpoint,
    router_builder: RouterBuilder,
    mut command_receiver: CommandReceiver,
) -> anyhow::Result<()> {
    let data = Data::new_memory();

    let (router_builder, _gossip) = setup_gossip(router_builder, endpoint.clone());

    let _router = router_builder.spawn();

    // TODO we should store additional known peers somewhere..
    let mut bootstrap = vec![endpoint.id()];

    // Guest Instances
    let mut instance_map: InstanceMap = BTreeMap::new();
    //
    while let Some(cmd) = command_receiver.recv().await {
        let res = match cmd {
            Commands::CreateModule(create_module) => {
                handle_create_module(&data, create_module, bootstrap.clone(), &mut instance_map)
                    .await
            }
            Commands::UpdateBootstrap(update_bootstrap) => {
                handle_update_bootstrap(update_bootstrap, &mut bootstrap).await
            }
            Commands::UpdateModule(update_module) => {
                handle_update_module(&data, update_module, &mut instance_map, bootstrap.clone())
                    .await
            }
        };
    }

    Ok(())
}
