use extism::{PluginBuilder, UserData};
use iroh::{Endpoint, EndpointId, protocol::RouterBuilder};
use iroh_gossip::{ALPN, Gossip};
use log::info;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

const GLOBAL_TOPIC: &str = "fern-global";

type SendChannel = tokio::sync::mpsc::Sender<()>;
type RecvChannel = tokio::sync::mpsc::Receiver<()>;

pub struct GuestGossip {
    gossip: Gossip,
    global_handle: JoinHandle<anyhow::Result<()>>,
}

pub fn attach_guest_gossip(
    plugin: PluginBuilder,
    mut router: RouterBuilder,
    endpoint: Endpoint,
    bootstrap: Vec<EndpointId>,
) -> (PluginBuilder, RouterBuilder, UserData<GuestGossip>) {
    let gossip = Gossip::builder().spawn(endpoint.clone());

    router = router.accept(ALPN, gossip.clone());

    let (tx, rx) = tokio::sync::mpsc::channel(1000);
    let global_handle =
        tokio::task::spawn(plugin_global_gossip_task(gossip.clone(), tx, rx, bootstrap));

    let gossip = UserData::new(GuestGossip {
        gossip,
        global_handle,
    });

    (plugin, router, gossip)
}

async fn plugin_global_gossip_task(
    gossip: Gossip,
    send_channel: SendChannel,
    recv_channel: RecvChannel,
    bootstrap: Vec<EndpointId>,
) -> anyhow::Result<()> {
    let global_topic = hmac_sha256::Hash::hash(GLOBAL_TOPIC.as_bytes());
    let mut started = gossip.subscribe(global_topic.into(), bootstrap).await?;

    info!("Gossip waiting for first peer connection");
    // Wait until we've connected to at least one peer
    started.joined().await?;
    info!("Gossip has connected to a peer");

    let (global_tx, mut global_rx) = started.split();

    // Kick task to broadcast messages from guest
    tokio::task::spawn(async move {
        let mut recv_channel = recv_channel;
        let global_tx = global_tx;

        while let Some(msg) = recv_channel.recv().await {
            let bytes = serde_json::to_vec(&msg).unwrap();
            let res = global_tx.broadcast(bytes.into()).await;
            info!("guest gossip broadcast res {res:?}")
        }
    });

    while let Some(Ok(next)) = global_rx.next().await {
        match next {
            iroh_gossip::api::Event::Received(message) => {
                if let Ok(msg) = serde_json::from_slice(&message.content) {
                    let res = send_channel.send(msg).await;
                }
            }
            event => {
                info!("guest gossip event {event:?}");
            }
        }
    }
    Ok(())
}
