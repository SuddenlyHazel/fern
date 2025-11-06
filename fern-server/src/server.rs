use std::{collections::BTreeMap, sync::Arc, thread, time::Duration};

use iroh::{
    Endpoint,
    discovery::dns::DnsDiscovery,
    protocol::{Router, RouterBuilder},
};
use log::info;
use tokio::{
    sync::mpsc,
    task::{JoinHandle, LocalSet},
};

use crate::{
    data::Data, guest_instance::GuestInstance, server::get_info::handle_get_info,
    server::gossip::setup_gossip,
};

pub mod create_module;
pub use create_module::*;

pub mod update_bootstrap;
pub use update_bootstrap::*;

pub mod update_module;
pub use update_module::*;

pub mod gossip;

pub mod get_info;
pub use get_info::*;

pub enum Commands {
    CreateModule(CreateModule),
    UpdateBootstrap(UpdateBootstrap),
    UpdateModule(UpdateModule),
    GetInfo(GetInfo),
}

pub type CommandReceiver = mpsc::Receiver<Commands>;
pub type CommandSender = mpsc::Sender<Commands>;
pub type InstanceMap = BTreeMap<String, GuestInstance>;

pub struct ServerBuilder {
    sender: CommandSender,
    receiver: CommandReceiver,
    endpoint: Endpoint,
    router_builder: RouterBuilder,
}

impl ServerBuilder {
    pub fn start(self) -> Server {
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            let local_set = LocalSet::new();

            local_set.block_on(&rt, async move {
                let task = Server::start(self.endpoint, self.router_builder, self.receiver);
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
        });

        Server {
            sender: self.sender,
            //task
        }
    }
}
#[derive(Clone)]
pub struct Server {
    sender: CommandSender,
    //task: Arc<JoinHandle<anyhow::Result<()>>>,
}

impl Server {
    pub async fn builder() -> ServerBuilder {
        let endpoint = Endpoint::builder()
            //.discovery(DnsDiscovery::n0_dns())
            //.secret_key(secret)
            .bind()
            .await
            .expect("failed to bind iroh endpoint");

        let router_builder = Router::builder(endpoint.clone());

        let (sender, receiver) = mpsc::channel(100);
        ServerBuilder {
            sender,
            receiver,
            endpoint,
            router_builder,
        }
    }

    fn start(
        endpoint: Endpoint,
        router_builder: RouterBuilder,
        receiver: CommandReceiver,
    ) -> Arc<JoinHandle<anyhow::Result<()>>> {
        Arc::new(tokio::task::spawn_local(server_task(
            endpoint,
            router_builder,
            receiver,
        )))
    }

    pub fn new(endpoint: Endpoint, router_builder: RouterBuilder) -> Self {
        let (sender, rx) = mpsc::channel(100);

        let task = Arc::new(tokio::task::spawn_local(server_task(
            endpoint,
            router_builder,
            rx,
        )));

        Self { sender }
    }
}

pub async fn server_task(
    endpoint: Endpoint,
    router_builder: RouterBuilder,
    mut command_receiver: CommandReceiver,
) -> anyhow::Result<()> {
    info!("Starting Fern 🌿 Server");
    let data = Data::new_memory();
    let (router_builder, _gossip) = setup_gossip(router_builder, endpoint.clone());
    let _router = router_builder.spawn();

    // TODO we should store additional known peers somewhere..
    let mut bootstrap = vec![endpoint.id()];

    // Guest Instances
    let mut instance_map: InstanceMap = BTreeMap::new();
    //

    info!("Entering server event loop");
    while let Some(cmd) = command_receiver.recv().await {
        let res = match cmd {
            Commands::CreateModule(create_module) => {
                info!("Processing CreateModule Command");
                handle_create_module(&data, create_module, bootstrap.clone(), &mut instance_map)
                    .await
            }
            Commands::UpdateBootstrap(update_bootstrap) => {
                info!("Processing UpdateBootstrap Command");
                handle_update_bootstrap(update_bootstrap, &mut bootstrap).await
            }
            Commands::UpdateModule(update_module) => {
                info!("Processing UpdateModule Command");
                handle_update_module(&data, update_module, &mut instance_map, bootstrap.clone())
                    .await
            }
            Commands::GetInfo(get_info) => {
                info!("Processing GetInfo Command");
                handle_get_info(get_info, &endpoint, &instance_map).await
            }
        };
        info!("command outcome {res:?}");
    }

    Ok(())
}
