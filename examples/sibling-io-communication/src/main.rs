use anyhow::Ok;
use integration_tests_sv2::start_template_provider;
use mining_server_handler::MyMiningServerHandler;
use template_distribution_handler::MyTemplateDistributionHandler;
use tower_stratum::{
    client::service::{
        Sv2ClientService, request::RequestToSv2Client,
        subprotocols::mining::handler::NullSv2MiningClientHandler,
        subprotocols::template_distribution::trigger::TemplateDistributionClientTrigger,
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
    let mut client_service = Sv2ClientService::new_from_sibling_io(
        client_config,
        NullSv2MiningClientHandler,
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
            TemplateDistributionClientTrigger::SetCoinbaseOutputConstraints(
                config
                    .client_config
                    .template_distribution_config
                    .as_ref()
                    .unwrap()
                    .coinbase_output_constraints
                    .0,
                config
                    .client_config
                    .template_distribution_config
                    .as_ref()
                    .unwrap()
                    .coinbase_output_constraints
                    .1,
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
