use std::collections::btree_map::Entry;

use anyhow::anyhow;
use fern_runtime::{guest::new_guest, iroh_helpers::iroh_bundle};
use iroh::EndpointId;
use tokio::sync::oneshot;

use crate::{
    data::{Data, GuestRow},
    guest_instance::GuestInstance,
    server::{InstanceMap, Server},
};

pub struct CreateModule {
    name: String,
    module: Vec<u8>,
    reply: oneshot::Sender<CreateResponse>,
}

impl Server {
    pub async fn create_module(
        &self,
        name: String,
        module: Vec<u8>,
    ) -> anyhow::Result<CreateResponse> {
        let (tx, rx) = oneshot::channel();
        let cmd = CreateModule {
            name,
            module,
            reply: tx,
        };

        self.sender.send(super::Commands::CreateModule(cmd)).await?;

        Ok(rx.await?)
    }
}

pub struct CreateResponse {
    pub endpoint_id: EndpointId,
}

pub(crate) async fn handle_create_module(
    data: &Data,
    cmd: CreateModule,
    bootstrap: Vec<EndpointId>,
    instance_map: &mut InstanceMap,
) -> anyhow::Result<()> {
    let entry = match instance_map.entry(cmd.name.clone()) {
        Entry::Vacant(vacant_entry) => vacant_entry,
        Entry::Occupied(_) => {
            return Err(anyhow!("Module with name {} already exists", cmd.name));
        }
    };

    let (endpoint, router_builder) = iroh_bundle().await?;
    let guest_row = GuestRow::create(data, cmd.name, cmd.module)?;

    let guest = new_guest(guest_row.module, (endpoint, router_builder, bootstrap))?;

    let guest_instance = GuestInstance::new(guest);
    let endpoint_id = guest_instance.node_id();

    entry.insert(guest_instance);

    cmd.reply
        .send(CreateResponse { endpoint_id })
        .map_err(|_| anyhow::anyhow!("Failed to send response"))?;
    Ok(())
}
