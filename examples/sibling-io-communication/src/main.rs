use anyhow::Ok;
use integration_tests_sv2::start_template_provider;
use mining_server_handler::MyMiningServerHandler;
use template_distribution_handler::MyTemplateDistributionHandler;
use tower_stratum::{
    client::service::{
        Sv2ClientService, request::RequestToSv2Client,
        subprotocols::template_distribution::request::RequestToSv2TemplateDistributionClientService,
    },
    server::service::Sv2ServerService,
    tower::Service,
};
use tracing::info;
mod configs;
mod mining_server_handler;
mod template_distribution_handler;

// This example demonstrates how to use the SiblingIO feature described in the README.
// The goal is to spawn three services:
// 1. A Template Provider that sends new templates.
// 2. A client that connects to the Template Provider and receives these templates.
// 3. A server that receives the templates from the client via SiblingIO.
//
// The `template_distribution_handler.rs` file sends templates to `mining_server_handler.rs` using SiblingIO.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let mut config = configs::MyConfig::new();

    // Create a Template Provider that will accept connections from the TemplateDistributionClientHandler.
    // This simulates a Template Provider used to send templates.
    // In a real-world scenario, this would be a separate process or service.
    // Typically, a Bitcoin node with SV2 support would be used.
    // For this example, we use a fork maintained by Sjors: https://github.com/Sjors/bitcoin/releases.
    let (_tp, tp_address) = start_template_provider(None);

    // Since the Template Provider created by `start_template_provider` listens on a dynamic port,
    // we need to update our configuration with the correct address.
    config
        .client_config
        .template_distribution_config
        .as_mut()
        .unwrap()
        .server_addr = tp_address;

    info!("Template Provider address: {:?}", tp_address);

    // Initialize the handlers for TemplateDistribution and MiningServer.
    let tdc_handler = MyTemplateDistributionHandler::default();
    let mining_handler = MyMiningServerHandler::default();

    // Create the Sv2ServerService and Sv2ClientService using the handlers.

    // The Sv2ServerService returns a [`Sv2SiblingServerServiceIo`] object.
    // This object is used to send and receive requests to/from a sibling [`tower_stratum::client::service::Sv2ClientService`].
    let (
        mut server_service,
        sibling_server_io, // <----- SiblingIO is created here.
    ) = Sv2ServerService::new_with_sibling_io(config.server_config, mining_handler).unwrap();

    // Create the client service that communicates with the server using the `sibling_server_io` created above.
    let client_config = config.client_config.clone();
    let mut client_service = Sv2ClientService::new_with_sibling_io(
        client_config,
        tdc_handler,
        sibling_server_io, // <----- SiblingIO is passed here.
    )?;

    // Start the server and client services.
    server_service.start().await?;
    client_service.start().await?;

    // Once the connection is established, set the coinbase constraints with the Template Provider.
    // This step is necessary to start receiving new templates.
    client_service
        .call(RequestToSv2Client::TemplateDistributionTrigger(
            RequestToSv2TemplateDistributionClientService::SetCoinbaseOutputConstraints(
                config
                    .client_config
                    .template_distribution_config
                    .as_ref()
                    .unwrap()
                    .coinbase_output_constraints
                    .0
                    .clone(),
                config
                    .client_config
                    .template_distribution_config
                    .as_ref()
                    .unwrap()
                    .coinbase_output_constraints
                    .1
                    .clone(),
            ),
        ))
        .await
        .unwrap();

    // At this point, the client starts receiving new templates from the Template Provider.
    // These templates are sent to the MiningServer via SiblingIO.
    // Check the handlers to see how the messages are passed.
    // Logs are printed in the handlers for debugging and monitoring.

    // Wait for a Ctrl-C signal to terminate the application.
    tokio::signal::ctrl_c().await?;

    // Shutdown the server and client services gracefully.
    server_service.shutdown().await;
    client_service.shutdown().await;
    info!("Server and Client services shutdown");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, str::FromStr};

    use const_sv2::MESSAGE_TYPE_COINBASE_OUTPUT_CONSTRAINTS;
    use integration_tests_sv2::{sniffer::MessageDirection, start_sniffer};
    use roles_logic_sv2::template_distribution_sv2::{NewTemplate, SetNewPrevHash};
    use tower_stratum::{
        client::service::{request::RequestToSv2ClientError, response::ResponseFromSv2Client},
        server::service::{
            request::RequestToSv2Server, subprotocols::mining::request::RequestToSv2MiningServer,
        },
    };

    use super::*;

    #[tokio::test]
    async fn test_sibling_io() {
        // Initialize the configuration for the test.
        let mut config = configs::MyConfig::new();

        // Start a Template Provider that simulates a Bitcoin node with SV2 support.
        let (_tp, tp_address) = start_template_provider(None);

        // Start a sniffer to intercept messages between the client and the Template Provider.
        let (tp_sniffer, tp_sniffer_addr) =
            start_sniffer("".to_string(), tp_address, false, None).await;

        // Update the client configuration to use the sniffer's address.
        config
            .client_config
            .template_distribution_config
            .as_mut()
            .unwrap()
            .server_addr = tp_sniffer_addr;

        config.server_config.tcp_config.listen_address =
            SocketAddr::from_str("127.0.0.1:0").unwrap();

        // Allow some time for the sniffer to initialize.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Initialize the handlers for TemplateDistribution and MiningServer.
        let tdc_handler = MyTemplateDistributionHandler::default();
        let mining_handler = MyMiningServerHandler::default();

        // Create the Sv2ServerService and its sibling IO for communication with the client.
        let (
            mut server_service,
            sibling_server_io, // SiblingIO is created here.
        ) = Sv2ServerService::new_with_sibling_io(config.server_config, mining_handler).unwrap();

        // Create the Sv2ClientService and connect it to the server using the sibling IO.
        let client_config = config.client_config.clone();
        let mut client_service =
            Sv2ClientService::new_with_sibling_io(client_config, tdc_handler, sibling_server_io)
                .unwrap();

        // Start the server and client services.
        server_service.start().await.unwrap();
        client_service.start().await.unwrap();

        // Trigger the Template Provider to set coinbase output constraints.
        client_service
            .call(RequestToSv2Client::TemplateDistributionTrigger(
                RequestToSv2TemplateDistributionClientService::SetCoinbaseOutputConstraints(
                    config
                        .client_config
                        .template_distribution_config
                        .as_ref()
                        .unwrap()
                        .coinbase_output_constraints
                        .0
                        .clone(),
                    config
                        .client_config
                        .template_distribution_config
                        .as_ref()
                        .unwrap()
                        .coinbase_output_constraints
                        .1
                        .clone(),
                ),
            ))
            .await
            .unwrap();

        // Wait for the sniffer to detect the coinbase output constraints message.
        tp_sniffer
            .wait_for_message_type(
                MessageDirection::ToUpstream,
                MESSAGE_TYPE_COINBASE_OUTPUT_CONSTRAINTS,
            )
            .await;

        // Create a dummy NewTemplate message to simulate a new mining template.
        let new_template = NewTemplate {
            template_id: 0,
            future_template: false,
            version: 0,
            coinbase_tx_version: 0,
            coinbase_prefix: binary_codec_sv2::B0255::Owned(hex::decode("00").unwrap().to_vec()),
            coinbase_tx_input_sequence: 0,
            coinbase_tx_value_remaining: 0,
            coinbase_tx_outputs_count: 0,
            coinbase_tx_outputs: binary_codec_sv2::B064K::Owned(
                hex::decode("00").unwrap().to_vec(),
            ),
            coinbase_tx_locktime: 0,
            merkle_path: binary_codec_sv2::Seq0255::new(Vec::new()).unwrap(),
        };

        // Send the NewTemplate message to the sibling server and verify the response.
        let new_template_response = client_service
            .call(RequestToSv2Client::SendRequestToSiblingServerService(
                Box::new(RequestToSv2Server::MiningTrigger(
                    RequestToSv2MiningServer::NewTemplate(new_template),
                )),
            ))
            .await;

        // Assert that the response was sent to the sibling server.
        assert!(matches!(
            new_template_response.as_ref().unwrap(),
            ResponseFromSv2Client::SentRequestToSiblingServerService(
                RequestToSv2Server::MiningTrigger(RequestToSv2MiningServer::NewTemplate(_))
            )
        ));

        // Create a dummy SetNewPrevHash message to simulate a new previous hash.
        let new_prev_hash = SetNewPrevHash {
            template_id: 0,
            prev_hash: binary_codec_sv2::U256::Owned(hex::decode("00").unwrap().to_vec()),
            header_timestamp: 0,
            n_bits: 0,
            target: binary_codec_sv2::U256::Owned(hex::decode("00").unwrap().to_vec()),
        };

        // Send the SetNewPrevHash message to the sibling server and verify the response.
        let new_prev_hash_response = client_service
            .call(RequestToSv2Client::SendRequestToSiblingServerService(
                Box::new(RequestToSv2Server::MiningTrigger(
                    RequestToSv2MiningServer::SetNewPrevHash(new_prev_hash),
                )),
            ))
            .await
            .unwrap();

        // Assert that the response from the MiningServerHandler is received.
        assert!(matches!(
            new_prev_hash_response,
            ResponseFromSv2Client::SentRequestToSiblingServerService(
                RequestToSv2Server::MiningTrigger(RequestToSv2MiningServer::SetNewPrevHash(_))
            )
        ));

        // Shutdown the server and client services gracefully.
        server_service.shutdown().await;
        client_service.shutdown().await;
    }

    #[tokio::test]
    async fn test_sibling_io_with_wrong_constructor() {
        // Initialize the configuration for the test.
        let mut config = configs::MyConfig::new();

        // Start a Template Provider that simulates a Bitcoin node with SV2 support.
        let (_tp, tp_address) = start_template_provider(None);

        // Update the client configuration to use the sniffer's address.
        config
            .client_config
            .template_distribution_config
            .as_mut()
            .unwrap()
            .server_addr = tp_address;

        config.server_config.tcp_config.listen_address =
            SocketAddr::from_str("127.0.0.1:0").unwrap();

        // Allow some time for the sniffer to initialize.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Initialize the handlers for TemplateDistribution and MiningServer.
        let tdc_handler = MyTemplateDistributionHandler::default();
        let mining_handler = MyMiningServerHandler::default();

        // Create the Sv2ServerService with a wrong constructor.
        let mut server_service =
            Sv2ServerService::new(config.server_config, mining_handler).unwrap();
        // Create the Sv2ClientService with a wrong constructor.
        let mut client_service = Sv2ClientService::new(config.client_config, tdc_handler).unwrap();

        // Start the server and client services.
        server_service.start().await.unwrap();
        client_service.start().await.unwrap();

        // Create a dummy NewTemplate message to simulate a new mining template.
        let new_template = NewTemplate {
            template_id: 0,
            future_template: false,
            version: 0,
            coinbase_tx_version: 0,
            coinbase_prefix: binary_codec_sv2::B0255::Owned(hex::decode("00").unwrap().to_vec()),
            coinbase_tx_input_sequence: 0,
            coinbase_tx_value_remaining: 0,
            coinbase_tx_outputs_count: 0,
            coinbase_tx_outputs: binary_codec_sv2::B064K::Owned(
                hex::decode("00").unwrap().to_vec(),
            ),
            coinbase_tx_locktime: 0,
            merkle_path: binary_codec_sv2::Seq0255::new(Vec::new()).unwrap(),
        };

        // Send the NewTemplate message to the sibling server and verify the response.
        let new_template_response = client_service
            .call(RequestToSv2Client::SendRequestToSiblingServerService(
                Box::new(RequestToSv2Server::MiningTrigger(
                    RequestToSv2MiningServer::NewTemplate(new_template),
                )),
            ))
            .await;

        // Assert that the error is of type RequestToSv2ClientError::NoSiblingServerServiceIo.
        assert!(matches!(
            new_template_response,
            Err(RequestToSv2ClientError::NoSiblingServerServiceIo)
        ));

        // Shutdown the server and client services gracefully.
        server_service.shutdown().await;
        client_service.shutdown().await;
    }
}
