use anyhow::anyhow;
use extism::{CurrentPlugin, FromBytes, Function, PTR, PluginBuilder, ToBytes, UserData, Val, ValType, host_fn, sdk::ExtismFunction};
use extism_convert::Json;
use iroh::{Endpoint, EndpointId, protocol::RouterBuilder};
use iroh_gossip::{ALPN, Gossip};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

const GLOBAL_TOPIC: &str = "fern-global";

type OutboundSendChannel = tokio::sync::mpsc::Sender<OutboundGossipMsg>;
type OutboundRecvChannel = tokio::sync::mpsc::Receiver<OutboundGossipMsg>;

type InboundSendChannel = tokio::sync::mpsc::Sender<InboundGossipMsg>;
type InboundRecvChannel = tokio::sync::mpsc::Receiver<InboundGossipMsg>;

pub struct GuestGossip {
    gossip: Gossip,
    global_handle: JoinHandle<anyhow::Result<()>>,
    // Transmits messages to the iroh gossip layer to be broadcast 
    // on the global gossip channel
    outbound_tx: OutboundSendChannel,
    // Receives messages from the iroh gossip layer to be passed to guest
    pub inbound_rx: InboundRecvChannel,
}

#[derive(Debug, Serialize, Deserialize, ToBytes)]
#[encoding(Json)]
pub struct InboundGossipMsg {
    pub topic: String,
    pub content: Value,
}

#[derive(Debug, Serialize, Deserialize, FromBytes)]
#[encoding(Json)]
pub struct OutboundGossipMsg {
    pub topic: String,
    pub content: Value,
}

pub fn attach_guest_gossip(
    plugin: PluginBuilder,
    mut router: RouterBuilder,
    endpoint: Endpoint,
    bootstrap: Vec<EndpointId>,
) -> (PluginBuilder, RouterBuilder, UserData<GuestGossip>) {

    // Global Gossip
    let gossip = Gossip::builder().spawn(endpoint.clone());

    router = router.accept(ALPN, gossip.clone());

    let (outbound_tx, outbound_rx) = tokio::sync::mpsc::channel(1000);
    let (inbound_tx, inbound_rx) = tokio::sync::mpsc::channel(1000);

    let global_handle = tokio::task::spawn(plugin_global_gossip_task(
        gossip.clone(),
        inbound_tx,
        outbound_rx,
        bootstrap,
    ));

    let gossip = UserData::new(GuestGossip {
        gossip,
        global_handle,
        outbound_tx,
        inbound_rx,
    });

    let plugin = plugin.with_function("broadcast_msg", [PTR], [PTR], gossip.clone(), broadcast_msg);
    
    (plugin, router, gossip)
}

async fn plugin_global_gossip_task(
    gossip: Gossip,
    inbound_tx: InboundSendChannel,
    outbound_rx: OutboundRecvChannel,
    bootstrap: Vec<EndpointId>,
) -> anyhow::Result<()> {
    let global_topic = hmac_sha256::Hash::hash(GLOBAL_TOPIC.as_bytes());
    let mut started = gossip.subscribe(global_topic.into(), bootstrap).await?;

    info!("Gossip waiting for first peer connection");
    // Wait until we've connected to at least one peer
    started.joined().await?;
    info!("Gossip has connected to a peer");

    let (global_tx, mut global_rx) = started.split();

    // Listen for guest broadcast requests and broadcast via iroh gossip
    tokio::task::spawn(async move {
        let mut recv_channel = outbound_rx;
        let global_tx = global_tx;

        while let Some(OutboundGossipMsg { topic, content }) = recv_channel.recv().await {
            let inbound_msg = InboundGossipMsg {
                topic,
                content,
            };
            let bytes = serde_json::to_vec(&inbound_msg).unwrap();
            let res = global_tx.broadcast(bytes.into()).await;
            info!("guest gossip broadcast res {res:?}")
        }
    });

    // Read incoming messages and send on inbound_tx to be passed to guest
    while let Some(Ok(next)) = global_rx.next().await {
        match next {
            iroh_gossip::api::Event::Received(message) => {
                if let Ok(msg) = serde_json::from_slice(&message.content) {
                    let res = inbound_tx.send(msg).await;
                }
            }
            event => {
                info!("guest gossip event {event:?}");
            }
        }
    }
    Ok(())
}

host_fn!(broadcast_msg(user_data: GuestGossip; msg: OutboundGossipMsg) -> () {
    execute_broadcast_msg(user_data, msg)
});

fn execute_broadcast_msg(
    user_data: UserData<GuestGossip>,
    msg: OutboundGossipMsg,
) -> Result<(), extism::Error> {
    let user_data = user_data.get()?;
    let locked = user_data.lock().unwrap();
    locked.outbound_tx.try_send(msg)?;
    Ok(())
}
