use fern_runtime::{guest::new_guest, iroh_helpers::iroh_bundle};
use iroh::{Endpoint, EndpointId, protocol::RouterBuilder};
use tokio::sync::{mpsc, oneshot};

use crate::{
    data::{Data, GuestRow},
    server::Server,
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
) -> anyhow::Result<()> {
    let (endpoint, router_builder) = iroh_bundle().await?;
    let guest_row = GuestRow::create(data, cmd.name, cmd.module)?;

    let guest = new_guest(guest_row.module, (endpoint, router_builder, bootstrap))?;
    cmd.reply
        .send(CreateResponse {
            endpoint_id: guest.get_node_id(),
        })
        .map_err(|_| anyhow::anyhow!("Failed to send response"))?;
    Ok(())
}
