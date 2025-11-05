use iroh::{Endpoint, SecretKey, discovery::dns::DnsDiscovery, protocol::RouterBuilder};

pub async fn iroh_bundle() -> anyhow::Result<(Endpoint, RouterBuilder)> {
    let endpoint = Endpoint::builder().discovery(DnsDiscovery::n0_dns().build());

    let endpoint = endpoint.bind().await?;

    let router = RouterBuilder::new(endpoint.clone());
    Ok((endpoint, router))
}

pub async fn iroh_bundle_with_secret(secret_key : SecretKey) -> anyhow::Result<(Endpoint, RouterBuilder)> {
    let endpoint = Endpoint::builder().secret_key(secret_key).discovery(DnsDiscovery::n0_dns().build());

    let endpoint = endpoint.bind().await?;

    let router = RouterBuilder::new(endpoint.clone());
    Ok((endpoint, router))
}

// #[tokio::test]
// async fn test() {
//     env_logger::builder()
//         .filter_level(log::LevelFilter::Info)
//         .init();
//     let e = iroh_bundle().await.unwrap();

//     tokio::time::sleep(std::time::Duration::from_secs(5)).await;
// }
