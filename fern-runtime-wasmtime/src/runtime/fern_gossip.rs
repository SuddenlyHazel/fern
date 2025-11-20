use std::{collections::HashSet, sync::atomic::AtomicBool};

use iroh::PublicKey;
use iroh_gossip::{Gossip, TopicId, api::GossipTopic};
use log::info;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_stream::StreamExt;

const GLOBAL_TOPIC_NAME: &str = "fern/global@0.1.0";

async fn start(
    joined_tx: tokio::sync::watch::Sender<bool>,
    topic: GossipTopic,
    out_rx: OutboundReceiver,
    in_rx: InboundSender,
) -> anyhow::Result<()> {
    let is_joined = topic.is_joined();

    info!("Starting runtime global gossip task is-joined: {is_joined}");
    joined_tx.send_replace(is_joined);

    let (topic_tx, mut topic_rx) = topic.split();

    while let Some(Ok(msg)) = topic_rx.next().await {
        info!("{msg:?}");
    }
    Ok(())
}

pub struct FernGossip {
    in_rx: InboundReceiver,
    out_tx: OutboundSender,
    joined_rx: tokio::sync::watch::Receiver<bool>,
}

pub struct Builder {
    gossip: Gossip,
    // Sender for task to publish gossip received over the wire
    in_tx: InboundSender,
    // Receiver for runtime to read gossip received over the wire
    in_rx: InboundReceiver,
    // Sender for runtime to produce messages to the gossip swarm
    out_tx: OutboundSender,
    // Receiver to Task for messages which will be published to the swarm
    out_rx: OutboundReceiver,
    peers: HashSet<PublicKey>,
    global_topic_name: String,
}

impl Builder {
    pub fn set_topic_name(mut self, topic: &str) -> Self {
        self.global_topic_name = topic.into();
        self
    }

    pub fn add_peers(mut self, peers: Vec<PublicKey>) -> Self {
        peers.into_iter().for_each(|v| {
            self.peers.insert(v);
        });
        self
    }

    pub fn add_peer(mut self, peer: PublicKey) -> Self {
        self.peers.insert(peer);
        self
    }

    pub async fn spawn(self) -> anyhow::Result<FernGossip> {
        let Builder {
            gossip,
            in_tx,
            in_rx,
            out_tx,
            out_rx,
            peers,
            global_topic_name,
        } = self;

        let gossip_sub = gossip
            .subscribe(
                TopicId::from_bytes(*blake3::hash(global_topic_name.as_bytes()).as_bytes()),
                peers.into_iter().collect(),
            )
            .await?;
        let (joined_tx, joined_rx) = tokio::sync::watch::channel(false);

        tokio::task::spawn(start(joined_tx, gossip_sub, out_rx, in_tx));
        Ok(FernGossip {
            in_rx,
            out_tx,
            joined_rx,
        })
    }
}

pub enum FernMsg {}

pub type InboundSender = Sender<FernMsg>;
pub type InboundReceiver = Receiver<FernMsg>;

pub type OutboundSender = Sender<FernMsg>;
pub type OutboundReceiver = Receiver<FernMsg>;

impl FernGossip {
    pub fn builder(gossip: Gossip) -> Builder {
        let (in_tx, in_rx) = tokio::sync::mpsc::channel::<FernMsg>(128);
        let (out_tx, out_rx) = tokio::sync::mpsc::channel::<FernMsg>(128);

        let peers = HashSet::new();
        let global_topic_name = GLOBAL_TOPIC_NAME.to_string();
        Builder {
            gossip,
            in_tx,
            in_rx,
            out_tx,
            out_rx,
            peers,
            global_topic_name,
        }
    }

    pub fn is_joined(&self) -> bool {
        *self.joined_rx.borrow()
    }
}
