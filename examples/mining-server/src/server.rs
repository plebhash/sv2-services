use crate::config::MyMiningServerConfig;
use crate::handler::MyMiningServerHandler;
use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio_util::sync::CancellationToken;
use tower_stratum::server::service::Sv2ServerService;
use tower_stratum::server::service::config::Sv2ServerServiceConfig;
use tower_stratum::server::service::config::Sv2ServerServiceMiningConfig;
use tower_stratum::server::service::config::Sv2ServerTcpConfig;
use tracing::info;

pub struct MyMiningServer {
    sv2_server_service: Sv2ServerService<MyMiningServerHandler>,
    cancellation_token: CancellationToken,
}

impl MyMiningServer {
    pub async fn new(config: MyMiningServerConfig) -> Result<Self> {
        let listen_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), config.listening_port);

        let tcp_config = Sv2ServerTcpConfig {
            listen_address,
            pub_key: config.pub_key,
            priv_key: config.priv_key,
            cert_validity: config.cert_validity,
        };

        let service_config = Sv2ServerServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            inactivity_limit: config.inactivity_limit,
            tcp_config,
            mining_config: Some(Sv2ServerServiceMiningConfig {
                supported_flags: 0b0101,
            }),
            job_declaration_config: None,
            template_distribution_config: None,
        };

        let cancellation_token = CancellationToken::new();

        let sv2_server_service = Sv2ServerService::new(
            service_config,
            MyMiningServerHandler::default(),
            cancellation_token.clone(),
        )?;
        Ok(MyMiningServer {
            sv2_server_service,
            cancellation_token,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.sv2_server_service.start().await?;
        info!("Mining server started");
        Ok(())
    }

    pub async fn shutdown(&mut self) {
        self.cancellation_token.cancel();
        info!("Mining server shutdown complete");
    }
}
