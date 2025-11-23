use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use extism::{Manifest, Plugin, PluginBuilder, UserData, Wasm};
use iroh::{
    Endpoint, EndpointId,
    protocol::{Router, RouterBuilder},
};
use tokio::runtime::Handle;

use crate::{
    guest_fns::{
        self,
        gossip::{GuestGossip, InboundGossipMsg},
        sqlite_improved::GuestSqliteDbImproved,
    },
    iroh_helpers::iroh_bundle,
};

const MESSAGE_FN: &str = "gossipMessageHandler";
const SQL_TEST: &str = "testEnhancedSql";
const SHUTDOWN_FN: &str = "shutdown";
const TICK_FN: &str = "tick";
const INIT_FN: &str = "init";

pub type IrohBundle = (Endpoint, RouterBuilder, Vec<EndpointId>);

#[derive(Default, Clone)]
pub struct GuestConfig {
    pub name: String,
    pub host_data_path: Option<PathBuf>,
}

pub struct Guest {
    pub plugin: Plugin,
    pub network_data: NetworkUserData,
    pub plugin_userdata: PluginUserData,
    pub router: Router,
    pub endpoint: Endpoint,
}

impl Guest {
    pub async fn tick_gossip(&mut self) -> anyhow::Result<()> {
        let msgs = {
            let network_data = self.network_data.gossip.get()?;
            // Forced scope to drop this fella
            let mut locked = network_data.try_lock().map_err(|e| anyhow!("{e}"))?;
            let mut msgs = vec![];
            while let Ok(msg) = locked.inbound_rx.try_recv() {
                msgs.push(msg);
            }
            msgs
        };

        for msg in msgs {
            // This kinda isn't great since the guest could be failing
            // but its better than nothing atm
            let _ = self.plugin.call::<InboundGossipMsg, ()>(MESSAGE_FN, msg);
        }

        Ok(())
    }

    pub fn initialize(&mut self) -> anyhow::Result<()> {
        Ok(self.plugin.call(INIT_FN, ())?)
    }

    pub fn shutdown(&mut self) -> anyhow::Result<()> {
        Ok(self.plugin.call(SHUTDOWN_FN, ())?)
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        Ok(self.plugin.call(TICK_FN, ())?)
    }

    pub fn get_node_id(&self) -> EndpointId {
        self.endpoint.id()
    }
}

pub struct NetworkUserData {
    pub gossip: UserData<GuestGossip>,
}

pub fn new_guest(
    config: GuestConfig,
    guest_module: impl Into<Wasm>,
    iroh: IrohBundle,
) -> anyhow::Result<Guest> {
    new_guest_with_userdata(config, guest_module, iroh, None)
}

pub fn new_guest_with_userdata(
    config: GuestConfig,
    guest_module: impl Into<Wasm>,
    iroh: IrohBundle,
    existing_user_data: Option<PluginUserData>,
) -> anyhow::Result<Guest> {
    let (plugin, plugin_userdata, Some((endpoint, router, bootstrap)), Some(network_data)) =
        new_plugin(config, guest_module, Some(iroh), existing_user_data)?
    else {
        return Err(anyhow!(
            "plugin didn't return iroh bundle and network stack. this should be impossible"
        ));
    };

    let router = router.spawn();
    Ok(Guest {
        plugin,
        network_data,
        plugin_userdata,
        router,
        endpoint,
    })
}

#[derive(Clone)]
pub struct PluginUserData {
    pub sqlite: UserData<GuestSqliteDbImproved>,
}

pub fn new_plugin(
    config: GuestConfig,
    guest_module: impl Into<Wasm>,
    mut iroh: Option<IrohBundle>,
    existing_user_data: Option<PluginUserData>,
) -> anyhow::Result<(
    Plugin,
    PluginUserData,
    Option<IrohBundle>,
    Option<NetworkUserData>,
)> {
    let manifest = Manifest::new([guest_module]).with_config_key("id", uuid::Uuid::new_v4());

    let builder = PluginBuilder::new(manifest).with_wasi(true);

    let builder = guest_fns::kv::attach_guest_kv(builder, config.clone());
    let (builder, sqlite) = guest_fns::sqlite_improved::attach_guest_sqlite_improved(
        builder,
        config.clone(),
        existing_user_data.as_ref().map(|ud| ud.sqlite.clone()),
    );
    let mut builder = guest_fns::debug::attach_guest_debug(builder);

    let mut network_user_data = None;
    if let Some((endpoint, router_builder, bootstrap)) = iroh {
        // TODO we need to return the gossip_user_data somehow
        let (new_builder, new_router, gossip_user_data) = guest_fns::gossip::attach_guest_gossip(
            builder,
            router_builder,
            endpoint.clone(),
            bootstrap.clone(),
        );

        iroh = Some((endpoint, new_router, bootstrap));

        network_user_data = Some(NetworkUserData {
            gossip: gossip_user_data,
        });
        builder = new_builder
    }
    let plugin = builder.build()?;

    let ud = PluginUserData { sqlite };
    Ok((plugin, ud, iroh, network_user_data))
}
