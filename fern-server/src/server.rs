use std::{collections::BTreeMap, path::{Path, PathBuf}, sync::Arc, thread, time::Duration};

use iroh::{
    Endpoint, PublicKey, SecretKey, discovery::dns::DnsDiscovery, protocol::{Router, RouterBuilder}
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::{
    signal::ctrl_c, sync::mpsc, task::{JoinHandle, LocalSet}
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

pub mod remove_module;
pub use remove_module::*;

pub mod gossip;

pub mod get_info;
pub use get_info::*;

// Not a command module
pub mod server_start;
pub use server_start::*;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub server_secret : Option<SecretKey>,
    pub db_path : Option<PathBuf>,
    pub guest_db_path : Option<PathBuf>,
}

pub enum Commands {
    CreateModule(CreateModule),
    UpdateBootstrap(UpdateBootstrap),
    UpdateModule(UpdateModule),
    RemoveModule(RemoveModule),
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
    config : Config,
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
                let _task = Server::start(self.endpoint, self.router_builder, self.receiver, self.config);
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
        });

        Server {
            sender: self.sender,
        }
    }

    pub fn with_secret(mut self, node_secret : SecretKey) -> Self {
        self.config.server_secret = Some(node_secret);
        self
    }

    pub fn with_db_path(mut self, db_path : &Path) -> Self {
        self.config.db_path = Some(db_path.into());
        self
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

        let config = Config::default();
        let (sender, receiver) = mpsc::channel(100);
        ServerBuilder {
            sender,
            receiver,
            endpoint,
            router_builder,
            config,
        }
    }

    fn start(
        endpoint: Endpoint,
        router_builder: RouterBuilder,
        receiver: CommandReceiver,
        config: Config,
    ) -> Arc<JoinHandle<anyhow::Result<()>>> {
        Arc::new(tokio::task::spawn_local(server_task(
            endpoint,
            router_builder,
            receiver,
            config,
        )))
    }

    pub fn new(endpoint: Endpoint, router_builder: RouterBuilder, config: Config) -> Self {
        let (sender, rx) = mpsc::channel(100);

        let task = Arc::new(tokio::task::spawn_local(server_task(
            endpoint,
            router_builder,
            rx,
            config,
        )));

        Self { sender }
    }
}

pub async fn server_task(
    endpoint: Endpoint,
    router_builder: RouterBuilder,
    mut command_receiver: CommandReceiver,
    config : Config,
) -> anyhow::Result<()> {
    let Config { db_path, guest_db_path,.. } = config;
    info!("Starting Fern 🌿 Server");

    let data = if let Some(db_path) = db_path {
        Data::new_path(&db_path)
    } else {
        warn!("Database path was not configured using in memory DB. All data will be lost!");
        Data::new_memory()
    };

    let (router_builder, _gossip) = setup_gossip(router_builder, endpoint.clone());
    let _router = router_builder.spawn();

    // TODO we should store additional known peers somewhere..
    let mut bootstrap = vec![endpoint.id()];

    // Guest Instances
    let mut instance_map: InstanceMap = BTreeMap::new();

    // Bring any existing guests back online
    handle_start_start(&data, bootstrap.clone(), &mut instance_map, guest_db_path.clone()).await?;

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
            Commands::RemoveModule(remove_module) => {
                info!("Processing RemoveModule Command");
                handle_remove_module(&data, remove_module, &mut instance_map).await
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