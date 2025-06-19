use crate::config::MyTemplateDistributionClientConfig;
use crate::handler::MyTemplateDistributionHandler;
use anyhow::{Result, anyhow};
use tower::ServiceExt;
use tower_stratum::client::service::Sv2ClientService;
use tower_stratum::client::service::config::Sv2ClientServiceConfig;
use tower_stratum::client::service::config::Sv2ClientServiceTemplateDistributionConfig;
use tower_stratum::client::service::subprotocols::mining::handler::NullSv2MiningClientHandler;
use tracing::info;

pub struct MyTemplateDistributionClient {
    sv2_client_service: Sv2ClientService<NullSv2MiningClientHandler, MyTemplateDistributionHandler>,
}

impl MyTemplateDistributionClient {
    pub async fn new(config: MyTemplateDistributionClientConfig) -> Result<Self> {
        let service_config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: None,
            endpoint_port: None,
            vendor: None,
            hardware_version: None,
            firmware: None,
            device_id: None,
            mining_config: None,
            job_declaration_config: None,
            template_distribution_config: Some(Sv2ClientServiceTemplateDistributionConfig {
                coinbase_output_constraints: (
                    config.coinbase_output_max_additional_size,
                    config.coinbase_output_max_additional_sigops,
                ),
                server_addr: config.server_addr,
                auth_pk: config.auth_pk,
                setup_connection_flags: 0, // no flags for setup_connection
            }),
        };

        // Create the handler instance
        let template_distribution_handler = MyTemplateDistributionHandler::new(
            config.coinbase_output_max_additional_size,
            config.coinbase_output_max_additional_sigops,
        );

        // Initialize the service with config and handler
        let sv2_client_service = Sv2ClientService::new(
            service_config,
            NullSv2MiningClientHandler,
            template_distribution_handler,
        )
        .map_err(|e| anyhow!("Failed to create Sv2ClientService: {:?}", e))?;

        Ok(Self { sv2_client_service })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.sv2_client_service
            .start()
            .await
            .map_err(|e| anyhow!("Failed to start Sv2ClientService: {:?}", e))?;

        self.sv2_client_service
            .ready()
            .await
            .map_err(|e| anyhow!("Service is not ready: {:?}", e))?;

        Ok(())
    }

    pub async fn shutdown(&mut self) {
        info!("Shutting down Template Distribution Client");
        self.sv2_client_service.shutdown().await;
    }
}
