use iroh::{Endpoint, PublicKey, SecretKey, discovery::dns::DnsDiscovery, protocol::Router};
use iroh_gossip::{ALPN as GOSSIP_ALPN, Gossip};
use log::info;
use owo_colors::OwoColorize;

#[derive(Clone)]
pub struct IrohBundle {
    pub endpoint: Endpoint,
    pub router: Router,
    pub gossip: Gossip,
}
pub async fn iroh_bundle(
    secret_key: SecretKey,
    bootrap_peers: Vec<PublicKey>,
) -> anyhow::Result<IrohBundle> {
    let mut endpoint = Endpoint::builder();

    // TODO allow users to overide these with
    // the config somehow
    #[cfg(debug_assertions)]
    {
        use iroh::discovery::mdns::MdnsDiscovery;
        info!("Running in debug mode. Using MdnsDiscovery");
        endpoint = endpoint.relay_mode(iroh::RelayMode::Disabled).discovery(
            MdnsDiscovery::builder()
                .service_name("fern-runtime")
                .build(secret_key.public())
                .expect("failed to setup local discovery for debug more run"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        info!("Using n0_dns discovery");
        endpoint = endpoint.discovery(DnsDiscovery::n0_dns());
    }

    let endpoint = endpoint.secret_key(secret_key).bind().await?;

    info!("Iroh Addrs {:?}", endpoint.addr());
    let mut router_builder = Router::builder(endpoint.clone());

    // Setup Gossip
    let gossip = Gossip::builder().alpn(GOSSIP_ALPN).spawn(endpoint.clone());

    router_builder = router_builder.accept(GOSSIP_ALPN, gossip.clone());

    let router = router_builder.spawn();

    println!("{}", "           Iroh Started!                 ".on_green());
    println!("{}", "✨ It's a long, long way to Ba Sing Se ✨".on_green());
    Ok(IrohBundle {
        endpoint,
        router,
        gossip,
    })
}
