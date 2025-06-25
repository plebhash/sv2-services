mod client;
mod config;
mod handler;

use crate::client::MyTemplateDistributionClient;
use crate::config::MyTemplateDistributionClientConfig;
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
    tracing_subscriber::fmt().init();

    // Parse command line arguments
    let args = Args::parse();

    // Load configuration from file
    let config = MyTemplateDistributionClientConfig::from_file(args.config)?;

    // Create and start the client
    let mut client = MyTemplateDistributionClient::new(config).await?;

    // Use tokio::select to wait for either client completion or Ctrl+C
    tokio::select! {
        result = client.start() => {
            if let Err(e) = result {
                tracing::error!("Client error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl+C, shutting down...");
        }
    }

    // Shutdown the client
    client.shutdown().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::client::MyTemplateDistributionClient;
    use crate::config::MyTemplateDistributionClientConfig;
    use integration_tests_sv2::{
        interceptor::MessageDirection, start_sniffer, start_template_provider,
    };
    use stratum_common::roles_logic_sv2::common_messages_sv2::*;
    use stratum_common::roles_logic_sv2::template_distribution_sv2::*;

    #[tokio::test]
    async fn test_template_distribution_client() {
        tracing_subscriber::fmt().init();

        let (_tp, tp_addr) = start_template_provider(None);

        let (sniffer, sniffer_addr) = start_sniffer("", tp_addr, false, vec![]);

        let config = MyTemplateDistributionClientConfig {
            server_addr: sniffer_addr,
            auth_pk: None,
            coinbase_output_max_additional_size: 1,
            coinbase_output_max_additional_sigops: 1,
        };

        // Give sniffer time to initialize
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let mut client = MyTemplateDistributionClient::new(config).await.unwrap();

        let mut client_clone = client.clone();
        tokio::spawn(async move {
            client_clone.start().await.unwrap();
        });

        // Wait for client to be ready
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        sniffer
            .wait_for_message_type(MessageDirection::ToUpstream, MESSAGE_TYPE_SETUP_CONNECTION)
            .await;
        sniffer
            .wait_for_message_type(
                MessageDirection::ToDownstream,
                MESSAGE_TYPE_SETUP_CONNECTION_SUCCESS,
            )
            .await;
        sniffer
            .wait_for_message_type(
                MessageDirection::ToUpstream,
                MESSAGE_TYPE_COINBASE_OUTPUT_CONSTRAINTS,
            )
            .await;

        sniffer
            .wait_for_message_type(MessageDirection::ToDownstream, MESSAGE_TYPE_NEW_TEMPLATE)
            .await;
        sniffer
            .wait_for_message_type(
                MessageDirection::ToDownstream,
                MESSAGE_TYPE_SET_NEW_PREV_HASH,
            )
            .await;

        client.shutdown().await;
    }
}
