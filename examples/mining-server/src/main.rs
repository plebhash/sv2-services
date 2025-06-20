mod client;
mod config;
mod handler;
mod server;

use crate::config::MyMiningServerConfig;
use crate::server::MyMiningServer;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
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
    let config = MyMiningServerConfig::from_file(args.config)?;

    // Create the server
    let mut server = MyMiningServer::new(config).await?;

    // Use tokio::select to wait for either server completion or Ctrl+C
    tokio::select! {
        result = server.start() => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl+C, shutting down...");
        }
    }

    // Shutdown the server
    server.shutdown().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use integration_tests_sv2::{
        interceptor::MessageDirection, start_mining_device_sv2, start_sniffer,
    };
    use roles_logic_sv2::common_messages_sv2::*;
    use roles_logic_sv2::mining_sv2::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_mining_server() {
        // Initialize logging
        tracing_subscriber::fmt().init();

        // Parse command line arguments
        let args = Args {
            config: PathBuf::from("config.toml"),
        };

        // Load configuration from file
        let config = MyMiningServerConfig::from_file(args.config).unwrap();

        // Create the server
        let mut server = MyMiningServer::new(config.clone()).await.unwrap();

        // Start the server in a background task
        let mut server_clone = server.clone();
        tokio::spawn(async move {
            if let Err(e) = server_clone.start().await {
                eprintln!("Server error: {}", e);
            }
        });

        // Wait for server to be ready
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Create the server address
        let server_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            config.listening_port,
        );

        // Start sniffer A
        let (sniffer_a, sniffer_a_addr) = start_sniffer("A", server_addr, false, vec![]);

        // Start sniffer B
        let (sniffer_b, sniffer_b_addr) = start_sniffer("B", server_addr, false, vec![]);

        // Start mining device A
        start_mining_device_sv2(
            sniffer_a_addr,
            None, // Don't pass the public key for testing
            None,
            None,
            0,
            None,
            true,
        );

        // Start mining device B
        start_mining_device_sv2(
            sniffer_b_addr,
            None, // Don't pass the public key for testing
            None,
            None,
            0,
            None,
            true,
        );

        // Wait for the setup connection success message
        sniffer_a
            .wait_for_message_type(
                MessageDirection::ToDownstream,
                MESSAGE_TYPE_SETUP_CONNECTION_SUCCESS,
            )
            .await;

        // Wait for the open standard mining channel message
        sniffer_a
            .wait_for_message_type(
                MessageDirection::ToUpstream,
                MESSAGE_TYPE_OPEN_STANDARD_MINING_CHANNEL,
            )
            .await;

        // Wait for the open standard mining channel success message
        sniffer_a
            .wait_for_message_type(
                MessageDirection::ToDownstream,
                MESSAGE_TYPE_OPEN_STANDARD_MINING_CHANNEL_SUCCESS,
            )
            .await;

        sniffer_b
            .wait_for_message_type(
                MessageDirection::ToDownstream,
                MESSAGE_TYPE_SETUP_CONNECTION_SUCCESS,
            )
            .await;

        sniffer_b
            .wait_for_message_type(
                MessageDirection::ToUpstream,
                MESSAGE_TYPE_OPEN_STANDARD_MINING_CHANNEL,
            )
            .await;

        sniffer_b
            .wait_for_message_type(
                MessageDirection::ToDownstream,
                MESSAGE_TYPE_OPEN_STANDARD_MINING_CHANNEL_SUCCESS,
            )
            .await;

        // Shutdown the server
        server.shutdown().await;
    }
}
