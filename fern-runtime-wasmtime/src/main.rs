use std::time::Duration;

use clap::Parser;
use fern_runtime_wasmtime::runtime::start;
use iroh::PublicKey;

#[derive(Parser, Debug)]
#[command(name = "fern-runtime")]
#[command(about = "Fern Runtime - WASM Component Runtime", long_about = None)]
struct Cli {
    /// Bootstrap peer public keys (can be specified multiple times)
    #[arg(short, long = "peer", value_name = "PUBLIC_KEY")]
    bootstrap_peers: Vec<PublicKey>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // This awaits until ctrl-c!
    Ok(start(None, cli.bootstrap_peers).await?)
}
