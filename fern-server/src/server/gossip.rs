use iroh::{Endpoint, protocol::RouterBuilder};
use iroh_gossip::{ALPN, Gossip};


pub fn setup_gossip(router_builder : RouterBuilder, endpoint: Endpoint) -> (RouterBuilder, Gossip) {
  let gossip = Gossip::builder().alpn(ALPN).spawn(endpoint);

  
  (router_builder.accept(ALPN, gossip.clone()), gossip)
}