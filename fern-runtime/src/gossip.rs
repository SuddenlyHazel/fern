use iroh::EndpointId;

use crate::guest_fns::gossip::InboundGossipMsg;

pub enum GossipMessage {
    GuestGossip(InboundGossipMsg),
    GuestCreated {
        node_id: EndpointId,
        name: String,
        hash: Vec<u8>,
    },
}
