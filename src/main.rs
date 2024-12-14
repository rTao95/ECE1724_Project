use clap::Parser;
use std::error::Error;
use tracing_subscriber::EnvFilter;

mod node;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Topic name for the Gossipsub network
    #[clap(short, long, default_value = "test-net")]
    topic: String,

    /// Password for authentication
    #[clap(short, long, default_value = "password")]
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let args = Args::parse();

    println!("Starting peer-to-peer system with topic: {}", args.topic);

    node::run_peer_to_peer_system(args.topic, args.password).await
}
