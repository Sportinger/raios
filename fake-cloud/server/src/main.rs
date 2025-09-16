use anyhow::Result;
use clap::Parser;
use fake_cloud_server::{run_server, ServerConfig};

#[derive(Parser, Debug)]
#[command(about = "Fake cloud control plane stub", version)]
struct Args {
    /// Listen address for the WebSocket server
    #[arg(long, default_value = "127.0.0.1:9001")]
    bind: String,
    /// Offline root public key (hex file) used to verify manifests
    #[arg(long)]
    root_pub: std::path::PathBuf,
    /// Optional registry root directory where verified artifacts are published
    #[arg(long)]
    registry: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let config = ServerConfig {
        bind: args.bind,
        root_pub: args.root_pub,
        registry: args.registry,
    };
    run_server(config).await
}
