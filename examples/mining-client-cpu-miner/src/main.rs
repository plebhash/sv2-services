mod client;
mod config;
mod handler;
mod miner;

use crate::client::MyMiningClient;
use crate::config::MyMiningClientConfig;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the TOML configuration file
    #[arg(short, long)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = Args::parse();

    // Load configuration from file
    let config = MyMiningClientConfig::from_file(args.config)?;

    // Create and start the client
    let mut client = MyMiningClient::new(config).await?;

    // Start the client service
    client.start().await?;

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    // Shutdown the client
    client.shutdown().await;

    Ok(())
}
