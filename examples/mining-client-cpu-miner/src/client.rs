use crate::config::MyMiningClientConfig;
use crate::handler::MyMiningClientHandler;
use anyhow::{Result, anyhow};
use tower_stratum::client::service::Sv2ClientService;
use tower_stratum::client::service::config::Sv2ClientServiceConfig;
use tower_stratum::client::service::config::Sv2ClientServiceMiningConfig;
use tower_stratum::client::service::subprotocols::template_distribution::handler::NullSv2TemplateDistributionClientHandler;
use tracing::info;

pub struct MyMiningClient {
    sv2_client_service:
        Sv2ClientService<MyMiningClientHandler, NullSv2TemplateDistributionClientHandler>,
}

impl MyMiningClient {
    pub async fn new(config: MyMiningClientConfig) -> Result<Self> {
        let service_config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: None,
            endpoint_port: None,
            vendor: None,
            hardware_version: None,
            firmware: None,
            device_id: None,
            mining_config: Some(Sv2ClientServiceMiningConfig {
                server_addr: config.server_addr,
                auth_pk: config.auth_pk,
                // REQUIRES_VERSION_ROLLING, !REQUIRES_WORK_SELECTION, REQUIRES_STANDARD_JOBS
                setup_connection_flags: 0b001_u32,
            }),
            job_declaration_config: None,
            template_distribution_config: None,
        };

        let template_distribution_handler = NullSv2TemplateDistributionClientHandler;

        let mining_handler = MyMiningClientHandler::new(
            config.user_identity,
            config.n_extended_channels,
            config.n_standard_channels,
        );

        let sv2_client_service = Sv2ClientService::new(
            service_config,
            mining_handler,
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

        Ok(())
    }

    pub async fn shutdown(&mut self) {
        info!("Shutting down Mining Client");
        self.sv2_client_service.shutdown().await;
    }
}
