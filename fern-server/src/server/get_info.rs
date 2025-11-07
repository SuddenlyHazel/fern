use iroh::{Endpoint, EndpointId};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::server::{InstanceMap, Server};

pub struct NodeAddress {
    pub reply: oneshot::Sender<NodeAddressResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestInfo {
    pub name: String,
    pub endpoint_id: EndpointId,
    pub module_hash: String,
}

pub struct Guests {
    pub reply: oneshot::Sender<Vec<GuestInfo>>,
}

pub struct NodeAddressResponse {
    address: EndpointId,
}
pub enum GetInfo {
    NodeAddress(NodeAddress),
    Guests(Guests),
}

impl Server {
    pub async fn node_address(&self) -> anyhow::Result<EndpointId> {
        let (tx, rx) = oneshot::channel();
        let cmd = NodeAddress { reply: tx };

        self.sender
            .send(super::Commands::GetInfo(GetInfo::NodeAddress(cmd)))
            .await?;

        Ok(rx.await?.address)
    }

    pub async fn guest_info(&self) -> anyhow::Result<Vec<GuestInfo>> {
        let (tx, rx) = oneshot::channel();
        let cmd = Guests { reply: tx };

        self.sender
            .send(super::Commands::GetInfo(GetInfo::Guests(cmd)))
            .await?;

        Ok(rx.await?)
    }
}

pub(crate) async fn handle_get_info(
    get_info: GetInfo,
    endpoint: &Endpoint,
    instance_map: &InstanceMap,
) -> anyhow::Result<()> {
    match get_info {
        GetInfo::NodeAddress(node_address) => handle_node_address(node_address, endpoint).await,
        GetInfo::Guests(guests) => handle_guests(guests, instance_map).await,
    }
}

pub(crate) async fn handle_node_address(
    cmd: NodeAddress,
    endpoint: &Endpoint,
) -> anyhow::Result<()> {
    let node_address = endpoint.id();

    let response = NodeAddressResponse {
        address: node_address,
    };

    // Send the response back through the oneshot channel
    if let Err(_) = cmd.reply.send(response) {
        log::warn!("Failed to send NodeAddress response - receiver dropped");
    }

    Ok(())
}

pub(crate) async fn handle_guests(cmd: Guests, instance_map: &InstanceMap) -> anyhow::Result<()> {
    let mut res = vec![];

    for (name, instance) in instance_map.iter() {
        res.push(GuestInfo {
            name: name.clone(),
            endpoint_id: instance.node_id(),
            module_hash: instance.module_hash.clone(),
        });
    }

    // Send the response back through the oneshot channel
    if let Err(_) = cmd.reply.send(res) {
        log::warn!("Failed to send GuestInfo response - receiver dropped");
    }

    Ok(())
}
