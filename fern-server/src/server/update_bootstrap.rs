use iroh::EndpointId;
use tokio::sync::oneshot;

use crate::server::Server;

pub struct UpdateBootstrap {
    pub nodes: Vec<EndpointId>,
    pub reply: oneshot::Sender<UpdateBootstrapResponse>,
}

impl Server {
    pub async fn update_bootstrap(
        &self,
        nodes: Vec<EndpointId>,
    ) -> anyhow::Result<UpdateBootstrapResponse> {
        let (tx, rx) = oneshot::channel();
        let cmd = UpdateBootstrap {
            nodes,
            reply: tx,
        };

        self.sender.send(super::Commands::UpdateBootstrap(cmd)).await?;

        Ok(rx.await?)
    }
}

pub struct UpdateBootstrapResponse {
    pub success: bool,
    pub node_count: usize,
}

pub(crate) async fn handle_update_bootstrap(
    cmd: UpdateBootstrap,
    bootstrap: &mut Vec<EndpointId>,
) -> anyhow::Result<()> {
    // Update the bootstrap vector with the new nodes
    *bootstrap = cmd.nodes.clone();
    
    cmd.reply
        .send(UpdateBootstrapResponse {
            success: true,
            node_count: cmd.nodes.len(),
        })
        .map_err(|_| anyhow::anyhow!("Failed to send response"))?;
    
    Ok(())
}