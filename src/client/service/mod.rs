use crate::client::service::config::Sv2ClientServiceConfig;
use crate::client::service::error::Sv2ClientServiceError;
use crate::client::service::event::{Sv2ClientEvent, Sv2ClientEventError};
use crate::client::service::outcome::Sv2ClientOutcome;
use crate::client::service::sibling::Sv2SiblingServerServiceIo;
use crate::client::service::subprotocols::mining::handler::NullSv2MiningClientHandler;
use crate::client::service::subprotocols::mining::handler::Sv2MiningClientHandler;
use crate::client::service::subprotocols::mining::trigger::MiningClientTrigger;
use crate::client::service::subprotocols::template_distribution::handler::NullSv2TemplateDistributionClientHandler;
use crate::client::service::subprotocols::template_distribution::handler::Sv2TemplateDistributionClientHandler;
use crate::client::service::subprotocols::template_distribution::trigger::TemplateDistributionClientTrigger;
use crate::client::tcp::encrypted::Sv2EncryptedTcpClient;
use crate::Sv2Service;
use async_channel::Receiver;
use std::future::Future;
use std::sync::Arc;
use stratum_common::roles_logic_sv2::common_messages_sv2::{Protocol, SetupConnection};
use stratum_common::roles_logic_sv2::mining_sv2::{
    OpenExtendedMiningChannel, OpenStandardMiningChannel,
};
use stratum_common::roles_logic_sv2::parsers_sv2::{
    AnyMessage, CommonMessages, Mining, TemplateDistribution,
};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

pub mod config;
pub mod error;
pub mod event;
pub mod outcome;
pub mod sibling;
pub mod subprotocols;

/// A [`Sv2Service`] implementer that provides:
/// - TCP connection to the server
/// - Connection management
/// - Optional handlers for the Mining, Job Declaration and Template Distribution protocols
/// - Ability to listen for messages from the server and trigger Service Events
///
/// The `M` generic paramenter is the handler for the Mining protocol.
/// If the service does not support the Mining protocol, it should be set to `NullSv2MiningClientHandler`.
///
/// The `J` generic paramenter is the handler for the Job Declaration protocol.
/// If the service does not support the Job Declaration protocol, it should be set to `NullSv2JobDeclarationClientHandler`.
///
/// The `T` generic paramenter is the handler for the Template Distribution protocol.
/// If the service does not support the Template Distribution protocol, it should be set to `NullSv2TemplateDistributionClientHandler`.
#[derive(Debug, Clone)]
pub struct Sv2ClientService<M, T>
// todo: add J generic parameter
where
    M: Sv2MiningClientHandler + Clone + Send + Sync + 'static,
    T: Sv2TemplateDistributionClientHandler + Clone + Send + Sync + 'static,
{
    config: Sv2ClientServiceConfig,
    mining_tcp_client: Arc<RwLock<Option<Sv2EncryptedTcpClient>>>,
    job_declaration_tcp_client: Arc<RwLock<Option<Sv2EncryptedTcpClient>>>,
    template_distribution_tcp_client: Arc<RwLock<Option<Sv2EncryptedTcpClient>>>,
    mining_handler: M,
    // todo: add job_declaration_handler: J,
    template_distribution_handler: T,
    cancellation_token: CancellationToken,
    sibling_server_service_io: Option<Sv2SiblingServerServiceIo>,
    event_injector: Option<Receiver<Sv2ClientEvent<'static>>>,
}

impl<M, T> Sv2ClientService<M, T>
where
    M: Sv2MiningClientHandler + Clone + Send + Sync + 'static,
    T: Sv2TemplateDistributionClientHandler + Clone + Send + Sync + 'static,
{
    /// Creates a new [`Sv2ClientService`]
    ///
    /// No sibling server service is required.
    pub fn new(
        config: Sv2ClientServiceConfig,
        mining_handler: M,
        template_distribution_handler: T,
        // todo: add job_declaration_handler: J,
        cancellation_token: CancellationToken,
    ) -> Result<Self, Sv2ClientServiceError> {
        let sv2_client_service = Self::_new(
            config,
            mining_handler,
            template_distribution_handler,
            None,
            None,
            cancellation_token,
        )?;
        Ok(sv2_client_service)
    }

    /// Creates a new [`Sv2ClientService`] with an event injector.
    ///
    /// The event injector is used to inject events into the client service.
    ///
    /// Can be used for example for handler functions that are not defined in the handler trait
    /// (and therefore cannot leverage `Sv2ClientOutcome::TriggerNewEvent`) but still need to send events to the server.
    ///
    /// Before calling this, you need to create a [`Receiver`] using [`async_channel::channel`].
    pub fn new_with_event_injector(
        config: Sv2ClientServiceConfig,
        mining_handler: M,
        template_distribution_handler: T,
        event_rx: Receiver<Sv2ClientEvent<'static>>,
        cancellation_token: CancellationToken,
    ) -> Result<Self, Sv2ClientServiceError> {
        let sv2_client_service = Self::_new(
            config,
            mining_handler,
            template_distribution_handler,
            None,
            Some(event_rx),
            cancellation_token,
        )?;
        Ok(sv2_client_service)
    }

    /// Creates a new [`Sv2ClientService`] with a sibling server service.
    ///
    /// The sibling server service is used to send and receive events to a sibling [`crate::server::service::Sv2ServerService`] that pairs with this client.    
    ///
    /// Before calling this, you need to create a [`Sv2SiblingServerServiceIo`] using [`crate::server::service::Sv2ServerService::new_with_sibling_io`].
    pub fn new_from_sibling_io(
        config: Sv2ClientServiceConfig,
        mining_handler: M,
        // todo: add job_declaration_handler: J,
        template_distribution_handler: T,
        sibling_server_service_io: Sv2SiblingServerServiceIo,
        cancellation_token: CancellationToken,
    ) -> Result<Self, Sv2ClientServiceError> {
        let sv2_client_service = Self::_new(
            config,
            mining_handler,
            template_distribution_handler,
            Some(sibling_server_service_io),
            None,
            cancellation_token,
        )?;
        Ok(sv2_client_service)
    }

    pub fn new_from_sibling_io_with_event_injector(
        config: Sv2ClientServiceConfig,
        mining_handler: M,
        // todo: add job_declaration_handler: J,
        template_distribution_handler: T,
        sibling_server_service_io: Sv2SiblingServerServiceIo,
        event_rx: Receiver<Sv2ClientEvent<'static>>,
        cancellation_token: CancellationToken,
    ) -> Result<Self, Sv2ClientServiceError> {
        let sv2_client_service = Self::_new(
            config,
            mining_handler,
            template_distribution_handler,
            Some(sibling_server_service_io),
            Some(event_rx),
            cancellation_token,
        )?;
        Ok(sv2_client_service)
    }

    // internal constructor
    fn _new(
        config: Sv2ClientServiceConfig,
        mining_handler: M,
        // todo: add job_declaration_handler: J,
        template_distribution_handler: T,
        sibling_server_service_io: Option<Sv2SiblingServerServiceIo>,
        event_injector: Option<Receiver<Sv2ClientEvent<'static>>>,
        cancellation_token: CancellationToken,
    ) -> Result<Self, Sv2ClientServiceError> {
        Self::validate_protocol_handlers(&config)?;

        let sv2_client_service = Sv2ClientService {
            config,
            mining_tcp_client: Arc::new(RwLock::new(None)),
            job_declaration_tcp_client: Arc::new(RwLock::new(None)),
            template_distribution_tcp_client: Arc::new(RwLock::new(None)),
            mining_handler,
            template_distribution_handler,
            cancellation_token,
            sibling_server_service_io,
            event_injector,
        };

        Ok(sv2_client_service)
    }

    // Validates that the protocol handlers are consistent with the supported protocols.
    // Returns an error if:
    // - A protocol is configured as supported but the corresponding handler is null.
    // - A protocol is not configured as supported but a non-null handler is provided.
    fn validate_protocol_handlers(
        config: &Sv2ClientServiceConfig,
    ) -> Result<(), Sv2ClientServiceError> {
        let supported_protocols: Vec<Protocol> = config
            .supported_protocols()
            .iter()
            .map(|(p, _)| *p)
            .collect();

        // Check if mining_handler is NullSv2MiningClientHandler
        let is_null_mining_handler = Self::has_null_handler(Protocol::MiningProtocol);

        if supported_protocols.contains(&Protocol::MiningProtocol) {
            if is_null_mining_handler {
                return Err(Sv2ClientServiceError::NullHandlerForSupportedProtocol {
                    protocol: Protocol::MiningProtocol,
                });
            }
        } else if !is_null_mining_handler {
            return Err(
                Sv2ClientServiceError::NonNullHandlerForUnsupportedProtocol {
                    protocol: Protocol::MiningProtocol,
                },
            );
        }

        // todo: add job declaration handler validation

        // Check if template_distribution_handler is NullSv2TemplateDistributionClientHandler
        let is_null_template_distribution_handler =
            Self::has_null_handler(Protocol::TemplateDistributionProtocol);

        // Check if template_distribution_handler is compatible with the supported protocols
        if supported_protocols.contains(&Protocol::TemplateDistributionProtocol) {
            if is_null_template_distribution_handler {
                return Err(Sv2ClientServiceError::NullHandlerForSupportedProtocol {
                    protocol: Protocol::TemplateDistributionProtocol,
                });
            }
        } else if !is_null_template_distribution_handler {
            return Err(
                Sv2ClientServiceError::NonNullHandlerForUnsupportedProtocol {
                    protocol: Protocol::TemplateDistributionProtocol,
                },
            );
        }
        Ok(())
    }

    /// Checks if the client is connected to the server
    pub async fn is_connected(&self, protocol: Protocol) -> bool {
        match protocol {
            Protocol::MiningProtocol => {
                let guard = self.mining_tcp_client.read().await;
                guard.is_some()
            }
            Protocol::JobDeclarationProtocol => {
                let guard = self.job_declaration_tcp_client.read().await;
                guard.is_some()
            }
            Protocol::TemplateDistributionProtocol => {
                let guard = self.template_distribution_tcp_client.read().await;
                guard.is_some()
            }
        }
    }

    pub async fn start(&mut self) -> Result<(), Sv2ClientServiceError> {
        for (protocol, flags) in self.config.supported_protocols() {
            let initiate_connection_outcome = self
                .handle(Sv2ClientEvent::SetupConnectionTrigger(protocol, flags))
                .await;
            match initiate_connection_outcome {
                Ok(Sv2ClientOutcome::Ok) => {
                    debug!("Connection established with {:?}", protocol);
                }
                _ => {
                    return Err(Sv2ClientServiceError::FailedToInitiateConnection { protocol });
                }
            }

            let mut this = self.clone();
            tokio::spawn(async move {
                if let Err(e) = this.listen_for_messages_via_tcp(protocol).await {
                    error!("Error listening for messages: {:?}", e);
                }
            });
        }

        let mut this = self.clone();
        if let Some(_sibling_server_service_io) = this.sibling_server_service_io.clone() {
            tokio::spawn(async move {
                if let Err(e) = this.listen_for_events_via_sibling_server_service().await {
                    error!("Error listening for event: {:?}", e);
                }
            });
        }

        let mut this = self.clone();
        if let Some(event_injector) = this.event_injector.clone() {
            tokio::spawn(async move {
                if let Err(e) = this
                    .listen_for_events_via_event_injector(event_injector)
                    .await
                {
                    error!("Error listening for event: {:?}", e);
                }
            });
        }

        if !Self::has_null_handler(Protocol::MiningProtocol) {
            match self
                .handle(Sv2ClientEvent::MiningTrigger(MiningClientTrigger::Start))
                .await
            {
                Ok(_) => {
                    debug!("Mining handler started");
                }
                Err(e) => {
                    error!("Failed to start mining handler: {:?}", e);
                    return Err(Sv2ClientServiceError::FailedToStartMiningHandler);
                }
            }
        }

        // todo: start job declaration handler

        if !Self::has_null_handler(Protocol::TemplateDistributionProtocol) {
            match self
                .handle(Sv2ClientEvent::TemplateDistributionTrigger(
                    TemplateDistributionClientTrigger::Start,
                ))
                .await
            {
                Ok(_) => {
                    debug!("Template distribution handler started");
                }
                Err(e) => {
                    error!("Failed to start template distribution handler: {:?}", e);
                    return Err(Sv2ClientServiceError::FailedToStartTemplateDistributionHandler);
                }
            }
        }

        // wait for cancellation token to be cancelled
        self.cancellation_token.cancelled().await;

        Ok(())
    }

    async fn initiate_connection(
        &mut self,
        protocol: Protocol,
        supported_flags: u32,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        // Establish TCP connection if not already connected
        let tcp_client = match protocol {
            Protocol::MiningProtocol => {
                if self.mining_tcp_client.read().await.is_none() {
                    let config = self.config.mining_config.as_ref().ok_or({
                        Sv2ClientEventError::UnsupportedProtocol {
                            protocol: Protocol::MiningProtocol,
                        }
                    })?;
                    let tcp_client = Sv2EncryptedTcpClient::new(config.server_addr, config.auth_pk)
                        .await
                        .ok_or_else(|| {
                            Sv2ClientEventError::ConnectionError(
                                "Failed to create TCP client".to_string(),
                            )
                        })?;
                    self.mining_tcp_client
                        .write()
                        .await
                        .replace(tcp_client.clone());
                    tcp_client
                } else {
                    self.mining_tcp_client
                        .read()
                        .await
                        .as_ref()
                        .expect("mining_tcp_client should be Some")
                        .clone()
                }
            }
            Protocol::JobDeclarationProtocol => {
                if self.job_declaration_tcp_client.read().await.is_none() {
                    let config = self.config.job_declaration_config.as_ref().ok_or({
                        Sv2ClientEventError::UnsupportedProtocol {
                            protocol: Protocol::JobDeclarationProtocol,
                        }
                    })?;
                    let tcp_client = Sv2EncryptedTcpClient::new(config.server_addr, config.auth_pk)
                        .await
                        .ok_or_else(|| {
                            Sv2ClientEventError::ConnectionError(
                                "Failed to create TCP client".to_string(),
                            )
                        })?;
                    self.job_declaration_tcp_client
                        .write()
                        .await
                        .replace(tcp_client.clone());
                    tcp_client
                } else {
                    self.job_declaration_tcp_client
                        .read()
                        .await
                        .as_ref()
                        .expect("job_declaration_tcp_client should be Some")
                        .clone()
                }
            }
            Protocol::TemplateDistributionProtocol => {
                if self.template_distribution_tcp_client.read().await.is_none() {
                    let config = self.config.template_distribution_config.as_ref().ok_or(
                        Sv2ClientEventError::UnsupportedProtocol {
                            protocol: Protocol::TemplateDistributionProtocol,
                        },
                    )?;
                    let tcp_client = Sv2EncryptedTcpClient::new(config.server_addr, config.auth_pk)
                        .await
                        .ok_or_else(|| {
                            Sv2ClientEventError::ConnectionError(
                                "Failed to create TCP client".to_string(),
                            )
                        })?;
                    self.template_distribution_tcp_client
                        .write()
                        .await
                        .replace(tcp_client.clone());
                    tcp_client
                } else {
                    self.template_distribution_tcp_client
                        .read()
                        .await
                        .as_ref()
                        .expect("template_distribution_tcp_client should be Some")
                        .clone()
                }
            }
        };

        let endpoint_host = self.config.endpoint_host.clone().unwrap_or_default();
        let endpoint_port = self.config.endpoint_port.unwrap_or_default();
        let vendor = self.config.vendor.clone().unwrap_or_default();
        let hardware_version = self.config.hardware_version.clone().unwrap_or_default();
        let firmware = self.config.firmware.clone().unwrap_or_default();
        let device_id = self.config.device_id.clone().unwrap_or_default();

        // Helper inline function to convert string to STR0_255
        let to_str0_255 = |s: String, field: &str| {
            s.clone().into_bytes().try_into().map_err(|_| {
                Sv2ClientEventError::StringConversionError(format!(
                    "Failed to convert {field} '{s}' to fixed-size array"
                ))
            })
        };

        let setup_connection = SetupConnection {
            protocol,
            min_version: self.config.min_supported_version,
            max_version: self.config.max_supported_version,
            flags: supported_flags,
            endpoint_host: to_str0_255(endpoint_host, "endpoint_host")?,
            endpoint_port,
            vendor: to_str0_255(vendor, "vendor")?,
            hardware_version: to_str0_255(hardware_version, "hardware_version")?,
            firmware: to_str0_255(firmware, "firmware")?,
            device_id: to_str0_255(device_id, "device_id")?,
        };

        // Send the setup connection message using the io field
        tcp_client.io.send_message(setup_connection.into()).await?;

        // wait for the server to respond with a SetupConnectionSuccess or SetupConnectionError
        // and return the appropriate outcome
        let message = tcp_client.io.recv_message().await?;
        match message {
            AnyMessage::Common(CommonMessages::SetupConnectionSuccess(
                setup_connection_success,
            )) => {
                let server_used_version = setup_connection_success.used_version;
                let server_used_flags = setup_connection_success.flags;

                debug!(
                    "SetupConnectionSuccess received: version: {}, flags: {}",
                    server_used_version, server_used_flags
                );
                Ok(Sv2ClientOutcome::Ok)
            }
            AnyMessage::Common(CommonMessages::SetupConnectionError(setup_connection_error)) => {
                let error_code =
                    String::from_utf8_lossy(setup_connection_error.error_code.inner_as_ref())
                        .to_string();
                Err(Sv2ClientEventError::SetupConnectionError(error_code))
            }
            _ => Err(Sv2ClientEventError::ConnectionError(
                "Unexpected message type in response to SetupConnection".to_string(),
            )),
        }
    }

    // Listens for messages from the server and triggers Service Events
    async fn listen_for_messages_via_tcp(
        &mut self,
        protocol: Protocol,
    ) -> Result<(), Sv2ClientServiceError> {
        if !self.is_connected(protocol).await {
            return Err(Sv2ClientServiceError::IsNotConnected);
        }

        let tcp_client: Sv2EncryptedTcpClient = match protocol {
            Protocol::MiningProtocol => match self.mining_tcp_client.read().await.as_ref() {
                Some(client) => client.clone(),
                None => return Err(Sv2ClientServiceError::IsNotConnected),
            },
            Protocol::JobDeclarationProtocol => {
                match self.job_declaration_tcp_client.read().await.as_ref() {
                    Some(client) => client.clone(),
                    None => return Err(Sv2ClientServiceError::IsNotConnected),
                }
            }
            Protocol::TemplateDistributionProtocol => {
                match self.template_distribution_tcp_client.read().await.as_ref() {
                    Some(client) => client.clone(),
                    None => return Err(Sv2ClientServiceError::IsNotConnected),
                }
            }
        };

        let cancellation_token = self.cancellation_token.clone();

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("Message listener task cancelled");
                    tcp_client.shutdown();
                    break;
                }
                message_result = tcp_client.io.recv_message() => {
                    match message_result {
                        Ok(message) => {
                            if let Err(e) = self.handle(Sv2ClientEvent::IncomingMessage(message)).await {
                                // this is a protection from attacks where a server sends a message that it knows the client cannot handle
                                // we simply log the error and ignore the message, without shutting down the client
                                error!("Error handling message: {:?}, message will be ignored", e);
                            }
                        }
                        Err(_) => {
                            error!("{:?} server closed the connection", protocol);
                            self.cancellation_token.cancel();
                            break;
                        }
                    }
                }
            }
        }

        // if the loop was cancelled, we need to remove the tcp client from the map
        match protocol {
            Protocol::MiningProtocol => {
                self.mining_tcp_client.write().await.take();
            }
            Protocol::JobDeclarationProtocol => {
                self.job_declaration_tcp_client.write().await.take();
            }
            Protocol::TemplateDistributionProtocol => {
                self.template_distribution_tcp_client.write().await.take();
            }
        }

        Ok(())
    }

    // Listens for events from the sibling server service and triggers Service Events
    async fn listen_for_events_via_sibling_server_service(
        &mut self,
    ) -> Result<(), Sv2ClientServiceError> {
        let sibling_server_service_io = self
            .sibling_server_service_io
            .as_ref()
            .ok_or(Sv2ClientServiceError::NoSiblingServerServiceIo)?;

        let cancellation_token = self.cancellation_token.clone();

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("Sibling server service event listener task cancelled");
                    break;
                }
                result = sibling_server_service_io.recv() => {
                    match result {
                        Ok(req) => {
                            debug!("Received event from sibling server service");

                            let mut service = self.clone();
                            if let Err(e) = service.handle(*req).await {
                                error!("Error handling event from sibling server service: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive event from sibling server service: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }

        debug!("Sibling server service event listener task ended");
        sibling_server_service_io.shutdown();
        Ok(())
    }

    // Listens for events from the event injector and triggers Service Events
    async fn listen_for_events_via_event_injector(
        &mut self,
        event_rx: Receiver<Sv2ClientEvent<'static>>,
    ) -> Result<(), Sv2ClientServiceError> {
        let cancellation_token = self.cancellation_token.clone();

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("Event injector listener task cancelled");
                    break;
                }
                result = event_rx.recv() => {
                    match result {
                        Ok(event) => {
                            debug!("Received event from event injector");
                            let mut service = self.clone();
                            if let Err(e) = service.handle(event.clone()).await {
                                error!("Error handling event from event injector: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive event from event injector: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }

        debug!("Event injector listener task ended");
        event_rx.close();

        Ok(())
    }

    /// Checks if the handler for the given protocol is a null handler
    fn has_null_handler(protocol: Protocol) -> bool {
        match protocol {
            Protocol::MiningProtocol => {
                std::any::TypeId::of::<M>() == std::any::TypeId::of::<NullSv2MiningClientHandler>()
            }
            Protocol::JobDeclarationProtocol => {
                todo!()
            }
            Protocol::TemplateDistributionProtocol => {
                std::any::TypeId::of::<T>()
                    == std::any::TypeId::of::<NullSv2TemplateDistributionClientHandler>()
            }
        }
    }
}

impl<M, T> Sv2Service for Sv2ClientService<M, T>
where
    M: Sv2MiningClientHandler + Clone + Send + Sync + 'static,
    T: Sv2TemplateDistributionClientHandler + Clone + Send + Sync + 'static,
{
    type Event = Sv2ClientEvent<'static>;
    type Outcome = Sv2ClientOutcome<'static>;
    type ServiceError = Sv2ClientServiceError;
    type EventError = Sv2ClientEventError;

    // we cannot use `async fn` syntax here because of the recursive calls to `self.handle`
    fn handle(
        &mut self,
        event: Sv2ClientEvent<'static>,
    ) -> impl Future<Output = Result<Sv2ClientOutcome<'static>, Sv2ClientEventError>> + Send {
        Box::pin(async move {
            let outcome = match event {
                Sv2ClientEvent::SetupConnectionTrigger(protocol, flags) => {
                    debug!("Sv2ClientService received a trigger event for initiating a connection under the {:?} protocol", protocol);
                    match protocol {
                        Protocol::MiningProtocol => {
                            if self.config.mining_config.is_none() {
                                return Err(Sv2ClientEventError::UnsupportedProtocol { protocol });
                            }
                        }
                        Protocol::JobDeclarationProtocol => {
                            if self.config.job_declaration_config.is_none() {
                                return Err(Sv2ClientEventError::UnsupportedProtocol { protocol });
                            }
                        }
                        Protocol::TemplateDistributionProtocol => {
                            if self.config.template_distribution_config.is_none() {
                                return Err(Sv2ClientEventError::UnsupportedProtocol { protocol });
                            }
                        }
                    }
                    self.initiate_connection(protocol, flags).await
                }
                Sv2ClientEvent::IncomingMessage(message) => {
                    match message {
                        AnyMessage::Common(common_message) => {
                            match common_message {
                                CommonMessages::SetupConnection(_) => {
                                    // a client should never receive a SetupConnection event
                                    error!("Sv2ClientService received a SetupConnection event");
                                    Err(Sv2ClientEventError::UnsupportedMessage)
                                }
                                CommonMessages::SetupConnectionSuccess(_) => {
                                    // the only situation where a client should receive a SetupConnectionSuccess is inside initiate_connection
                                    error!("Sv2ClientService received a SetupConnectionSuccess event outside of initiate_connection");
                                    Err(Sv2ClientEventError::BadRouting)
                                }
                                CommonMessages::SetupConnectionError(_) => {
                                    // the only situation where a client should receive a SetupConnectionError is inside initiate_connection
                                    error!("Sv2ClientService received a SetupConnectionError event outside of initiate_connection");
                                    Err(Sv2ClientEventError::BadRouting)
                                }
                                CommonMessages::ChannelEndpointChanged(_) => {
                                    // todo
                                    Ok(Sv2ClientOutcome::Ok)
                                }
                                CommonMessages::Reconnect(_) => {
                                    // todo
                                    Ok(Sv2ClientOutcome::Ok)
                                }
                            }
                        }
                        AnyMessage::TemplateDistribution(template_distribution_message) => {
                            // check if the template_distribution_handler is supported
                            if Self::has_null_handler(Protocol::TemplateDistributionProtocol) {
                                error!("Sv2ClientService received a TemplateDistribution message, but no template distribution handler is configured");
                                return Err(Sv2ClientEventError::UnsupportedProtocol {
                                    protocol: Protocol::TemplateDistributionProtocol,
                                });
                            }

                            match template_distribution_message {
                                TemplateDistribution::CoinbaseOutputConstraints(_) => {
                                    // a client should never receive a CoinbaseOutputConstraints message
                                    error!("Sv2ClientService received a CoinbaseOutputConstraints message");
                                    Err(Sv2ClientEventError::UnsupportedMessage)
                                }
                                TemplateDistribution::SubmitSolution(_) => {
                                    // a client should never receive a SubmitSolution event
                                    error!("Sv2ClientService received a SubmitSolution message");
                                    Err(Sv2ClientEventError::UnsupportedMessage)
                                }
                                TemplateDistribution::RequestTransactionData(_) => {
                                    // a client should never receive a RequestTransactionData event
                                    error!("Sv2ClientService received a RequestTransactionData message");
                                    Err(Sv2ClientEventError::UnsupportedMessage)
                                }
                                TemplateDistribution::NewTemplate(message) => {
                                    debug!("Sv2ClientService received a NewTemplate message");
                                    self.template_distribution_handler
                                        .handle_new_template(message)
                                        .await
                                }
                                TemplateDistribution::RequestTransactionDataError(message) => {
                                    debug!("Sv2ClientService received a RequestTransactionDataError message");
                                    self.template_distribution_handler
                                        .handle_request_transaction_data_error(message)
                                        .await
                                }
                                TemplateDistribution::RequestTransactionDataSuccess(message) => {
                                    debug!("Sv2ClientService received a RequestTransactionDataSuccess message");
                                    self.template_distribution_handler
                                        .handle_request_transaction_data_success(message)
                                        .await
                                }
                                TemplateDistribution::SetNewPrevHash(message) => {
                                    debug!("Sv2ClientService received a SetNewPrevHash message");
                                    self.template_distribution_handler
                                        .handle_set_new_prev_hash(message)
                                        .await
                                }
                            }
                        }
                        AnyMessage::Mining(mining_message) => {
                            // check if the mining_handler is supported
                            if Self::has_null_handler(Protocol::MiningProtocol) {
                                error!("Sv2ClientService received a Mining message, but no mining handler is configured");
                                return Err(Sv2ClientEventError::UnsupportedProtocol {
                                    protocol: Protocol::MiningProtocol,
                                });
                            }

                            match mining_message {
                                Mining::OpenStandardMiningChannel(_) => {
                                    // a client should never receive a OpenStandardMiningChannel message
                                    error!(
                                        "Sv2ClientService received a OpenStandardMiningChannel message"
                                    );
                                    Err(Sv2ClientEventError::UnsupportedMessage)
                                }
                                Mining::OpenExtendedMiningChannel(_) => {
                                    // a client should never receive a OpenExtendedMiningChannel message
                                    error!(
                                        "Sv2ClientService received a OpenExtendedMiningChannel message"
                                    );
                                    Err(Sv2ClientEventError::UnsupportedMessage)
                                }
                                Mining::UpdateChannel(_) => {
                                    // a client should never receive a UpdateChannel message
                                    error!("Sv2ClientService received a UpdateChannel message");
                                    Err(Sv2ClientEventError::UnsupportedMessage)
                                }
                                Mining::SubmitSharesStandard(_) => {
                                    // a client should never receive a SubmitSharesStandard message
                                    error!(
                                        "Sv2ClientService received a SubmitSharesStandard message"
                                    );
                                    Err(Sv2ClientEventError::UnsupportedMessage)
                                }
                                Mining::SubmitSharesExtended(_) => {
                                    // a client should never receive a SubmitSharesExtended message
                                    error!(
                                        "Sv2ClientService received a SubmitSharesExtended message"
                                    );
                                    Err(Sv2ClientEventError::UnsupportedMessage)
                                }
                                Mining::SetCustomMiningJob(_) => {
                                    // a client should never receive a SetCustomMiningJob message
                                    error!(
                                        "Sv2ClientService received a SetCustomMiningJob message"
                                    );
                                    Err(Sv2ClientEventError::UnsupportedMessage)
                                }
                                Mining::OpenStandardMiningChannelSuccess(success) => {
                                    debug!("Sv2ClientService received a OpenStandardMiningChannelSuccess message");
                                    self.mining_handler
                                        .handle_open_standard_mining_channel_success(success)
                                        .await
                                }
                                Mining::OpenExtendedMiningChannelSuccess(success) => {
                                    debug!("Sv2ClientService received a OpenExtendedMiningChannelSuccess message");
                                    self.mining_handler
                                        .handle_open_extended_mining_channel_success(success)
                                        .await
                                }
                                Mining::OpenMiningChannelError(error) => {
                                    debug!(
                                        "Sv2ClientService received a OpenMiningChannelError message"
                                    );
                                    self.mining_handler
                                        .handle_open_mining_channel_error(error)
                                        .await
                                }
                                Mining::UpdateChannelError(error) => {
                                    debug!(
                                        "Sv2ClientService received a UpdateChannelError message"
                                    );
                                    self.mining_handler.handle_update_channel_error(error).await
                                }
                                Mining::CloseChannel(close_channel) => {
                                    debug!("Sv2ClientService received a CloseChannel message");
                                    self.mining_handler
                                        .handle_close_channel(close_channel)
                                        .await
                                }
                                Mining::SetExtranoncePrefix(set_extranonce_prefix) => {
                                    debug!(
                                        "Sv2ClientService received a SetExtranoncePrefix message"
                                    );
                                    self.mining_handler
                                        .handle_set_extranonce_prefix(set_extranonce_prefix)
                                        .await
                                }
                                Mining::SubmitSharesSuccess(success) => {
                                    debug!(
                                        "Sv2ClientService received a SubmitSharesSuccess message"
                                    );
                                    self.mining_handler
                                        .handle_submit_shares_success(success)
                                        .await
                                }
                                Mining::SubmitSharesError(error) => {
                                    debug!("Sv2ClientService received a SubmitSharesError message");
                                    self.mining_handler.handle_submit_shares_error(error).await
                                }
                                Mining::NewMiningJob(new_mining_job) => {
                                    debug!("Sv2ClientService received a NewMiningJob message");
                                    self.mining_handler
                                        .handle_new_mining_job(new_mining_job)
                                        .await
                                }
                                Mining::NewExtendedMiningJob(new_extended_mining_job) => {
                                    debug!(
                                        "Sv2ClientService received a NewExtendedMiningJob message"
                                    );
                                    self.mining_handler
                                        .handle_new_extended_mining_job(new_extended_mining_job)
                                        .await
                                }
                                Mining::SetNewPrevHash(set_new_prev_hash) => {
                                    debug!("Sv2ClientService received a SetNewPrevHash message");
                                    self.mining_handler
                                        .handle_set_new_prev_hash(set_new_prev_hash)
                                        .await
                                }
                                Mining::SetCustomMiningJobSuccess(success) => {
                                    debug!(
                                        "Sv2ClientService received a SetCustomMiningJobSuccess message"
                                    );
                                    self.mining_handler
                                        .handle_set_custom_mining_job_success(success)
                                        .await
                                }
                                Mining::SetCustomMiningJobError(error) => {
                                    debug!(
                                        "Sv2ClientService received a SetCustomMiningJobError message"
                                    );
                                    self.mining_handler
                                        .handle_set_custom_mining_job_error(error)
                                        .await
                                }
                                Mining::SetTarget(set_target) => {
                                    debug!("Sv2ClientService received a SetTarget message");
                                    self.mining_handler.handle_set_target(set_target).await
                                }
                                Mining::SetGroupChannel(set_group_channel) => {
                                    debug!("Sv2ClientService received a SetGroupChannel message");
                                    self.mining_handler
                                        .handle_set_group_channel(set_group_channel)
                                        .await
                                }
                            }
                        }
                        AnyMessage::JobDeclaration(_job_declaration_message) => {
                            todo!()
                        }
                    }
                }
                Sv2ClientEvent::MiningTrigger(mining_trigger) => {
                    if Self::has_null_handler(Protocol::MiningProtocol) {
                        error!("Sv2ClientService received a MiningTrigger message, but no mining handler is configured");
                        return Err(Sv2ClientEventError::UnsupportedProtocol {
                            protocol: Protocol::MiningProtocol,
                        });
                    }

                    if !self.is_connected(Protocol::MiningProtocol).await {
                        error!("Sv2ClientService is not connected to the Mining protocol");
                        return Err(Sv2ClientEventError::IsNotConnected);
                    }

                    match mining_trigger {
                        MiningClientTrigger::Start => {
                            debug!("Sv2ClientService received a trigger event for starting the mining handler");
                            self.mining_handler.start().await
                        }
                        MiningClientTrigger::OpenStandardMiningChannel(
                            request_id,
                            user_identity,
                            nominal_hash_rate,
                            max_target,
                        ) => {
                            debug!("Sv2ClientService received a trigger event for sending OpenStandardMiningChannel");
                            let tcp_client = self
                                .mining_tcp_client
                                .read()
                                .await
                                .as_ref()
                                .expect("mining_tcp_client should be Some")
                                .clone();
                            let open_standard_mining_channel = AnyMessage::Mining(
                                Mining::OpenStandardMiningChannel(OpenStandardMiningChannel {
                                    request_id: request_id.into(),
                                    user_identity: user_identity
                                        .clone()
                                        .into_bytes()
                                        .try_into()
                                        .map_err(|_| {
                                            Sv2ClientEventError::StringConversionError(format!(
                                            "Failed to convert user_identity '{user_identity}' to fixed-size array"
                                        ))
                                        })?,
                                    nominal_hash_rate,
                                    max_target: max_target.try_into().map_err(|_| {
                                        Sv2ClientEventError::U256ConversionError(
                                            "Failed to convert max_target to fixed-size array".to_string(),
                                        )
                                    })?,
                                }),
                            );

                            let result = tcp_client
                                .io
                                .send_message(open_standard_mining_channel)
                                .await;
                            match result {
                                Ok(_) => {
                                    debug!("Successfully sent OpenStandardMiningChannel");
                                    Ok(Sv2ClientOutcome::Ok)
                                }
                                Err(e) => Err(e.into()),
                            }
                        }
                        MiningClientTrigger::OpenExtendedMiningChannel(
                            request_id,
                            user_identity,
                            nominal_hash_rate,
                            max_target,
                            min_rollable_extranonce_size,
                        ) => {
                            debug!("Sv2ClientService received a trigger event for sending OpenExtendedMiningChannel");
                            let tcp_client = self
                                .mining_tcp_client
                                .read()
                                .await
                                .as_ref()
                                .expect("mining_tcp_client should be Some")
                                .clone();

                            let open_extended_mining_channel = AnyMessage::Mining(
                                Mining::OpenExtendedMiningChannel(OpenExtendedMiningChannel {
                                    request_id,
                                    user_identity: user_identity
                                        .clone()
                                        .into_bytes()
                                        .try_into()
                                        .map_err(|_| {
                                            Sv2ClientEventError::StringConversionError(format!(
                                            "Failed to convert user_identity '{user_identity}' to fixed-size array",
                                        ))
                                        })?,
                                    nominal_hash_rate,
                                    max_target: max_target.try_into().map_err(|_| {
                                        Sv2ClientEventError::U256ConversionError(
                                            "Failed to convert max_target to fixed-size array".to_string(),
                                        )
                                    })?,
                                    min_extranonce_size: min_rollable_extranonce_size,
                                }),
                            );

                            let result = tcp_client
                                .io
                                .send_message(open_extended_mining_channel)
                                .await;
                            match result {
                                Ok(_) => {
                                    debug!("Successfully sent OpenExtendedMiningChannel");
                                    Ok(Sv2ClientOutcome::Ok)
                                }
                                Err(e) => Err(e.into()),
                            }
                        }
                    }
                }
                Sv2ClientEvent::TemplateDistributionTrigger(template_distribution_trigger) => {
                    // check if this service is configured for template distribution
                    if Self::has_null_handler(Protocol::TemplateDistributionProtocol) {
                        return Err(Sv2ClientEventError::UnsupportedProtocol {
                            protocol: Protocol::TemplateDistributionProtocol,
                        });
                    }
                    if !self
                        .is_connected(Protocol::TemplateDistributionProtocol)
                        .await
                    {
                        return Err(Sv2ClientEventError::IsNotConnected);
                    }
                    match template_distribution_trigger {
                        TemplateDistributionClientTrigger::Start => {
                            debug!("Sv2ClientService received a trigger event for starting the template distribution handler");
                            self.template_distribution_handler.start().await
                        }
                        TemplateDistributionClientTrigger::SetCoinbaseOutputConstraints(
                            max_additional_size,
                            max_additional_sigops,
                        ) => {
                            debug!("Sv2ClientService received a trigger event for sending CoinbaseOutputConstraints");
                            self.template_distribution_handler
                                .set_coinbase_output_constraints(
                                    max_additional_size,
                                    max_additional_sigops,
                                )
                                .await
                        }
                        TemplateDistributionClientTrigger::TransactionDataNeeded(_template_id) => {
                            debug!("Sv2ClientService received a trigger event for sending RequestTransactionData");
                            self.template_distribution_handler
                                .transaction_data_needed(_template_id)
                                .await
                        }
                        TemplateDistributionClientTrigger::SubmitSolution(submit_solution) => {
                            debug!("Sv2ClientService received a trigger event for sending SubmitSolution");
                            self.template_distribution_handler
                                .submit_solution(submit_solution)
                                .await
                        }
                    }
                }
                Sv2ClientEvent::SendEventToSiblingServerService(event) => {
                    debug!("Sv2ClientService received a SendEventToSiblingServerService event");
                    match self.sibling_server_service_io {
                        Some(ref io) => {
                            io.send(*event.clone()).map_err(|_| {
                                Sv2ClientEventError::FailedToSendEventToSiblingServerService
                            })?;
                            Ok(Sv2ClientOutcome::Ok)
                        }
                        None => {
                            error!("No sibling server service on Sv2ClientService");
                            Err(Sv2ClientEventError::NoSiblingServerServiceIo)
                        }
                    }
                }
                Sv2ClientEvent::SendMessageToMiningServer(message) => {
                    if Self::has_null_handler(Protocol::MiningProtocol) {
                        return Err(Sv2ClientEventError::UnsupportedProtocol {
                            protocol: Protocol::MiningProtocol,
                        });
                    }

                    if self.mining_tcp_client.read().await.is_none() {
                        return Err(Sv2ClientEventError::IsNotConnected);
                    }

                    let tcp_client = self
                        .mining_tcp_client
                        .read()
                        .await
                        .as_ref()
                        .expect("mining_tcp_client should be Some")
                        .clone();

                    match tcp_client
                        .io
                        .send_message(AnyMessage::Mining(*message))
                        .await
                    {
                        Ok(_) => {
                            debug!("Successfully sent message to mining server");
                            return Ok(Sv2ClientOutcome::Ok);
                        }
                        Err(e) => Err(e.into()),
                    }
                }
                Sv2ClientEvent::SendMessageToTemplateDistributionServer(message) => {
                    if Self::has_null_handler(Protocol::TemplateDistributionProtocol) {
                        return Err(Sv2ClientEventError::UnsupportedProtocol {
                            protocol: Protocol::TemplateDistributionProtocol,
                        });
                    }

                    if self.template_distribution_tcp_client.read().await.is_none() {
                        return Err(Sv2ClientEventError::IsNotConnected);
                    }

                    let tcp_client = self
                        .template_distribution_tcp_client
                        .read()
                        .await
                        .as_ref()
                        .expect("template_distribution_tcp_client should be Some")
                        .clone();

                    match tcp_client
                        .io
                        .send_message(AnyMessage::TemplateDistribution(*message))
                        .await
                    {
                        Ok(_) => {
                            debug!("Successfully sent message to template distribution server");
                            return Ok(Sv2ClientOutcome::Ok);
                        }
                        Err(e) => Err(e.into()),
                    }
                }
                // Sv2ClientEvent::SendMessageToJobDeclarationServer(message) => {
                //     if self.job_declaration_tcp_client.read().await.is_none() {
                //         return Err(Sv2ClientEventError::IsNotConnected);
                //     }

                //     let tcp_client = self.job_declaration_tcp_client.read().await.as_ref().expect("job_declaration_tcp_client should be Some").clone();

                //     match tcp_client.io.send_message(AnyMessage::JobDeclaration(message.0), message.1).await {
                //         Ok(_) => {
                //             debug!("Successfully sent message to job declaration server");
                //             return Ok(ResponseFromSv2Client::Ok);
                //         },
                //         Err(e) => Err(e.into()),
                //     }
                // }
                Sv2ClientEvent::MultipleEvents(events) => {
                    for event in events.as_ref() {
                        if let Err(e) = self.handle(event.clone()).await {
                            error!(
                                "Sv2ClientService::MultipleEvents {:?} generated an error {:?}",
                                event, e
                            );
                            return Err(e);
                        }
                    }
                    Ok(Sv2ClientOutcome::Ok)
                }
            };

            // allows for recursive chaining of events
            if let Ok(Sv2ClientOutcome::TriggerNewEvent(event)) = outcome {
                self.handle(*event).await
            } else {
                outcome
            }
        })
    }

    async fn start(&mut self) -> Result<(), Sv2ClientServiceError> {
        for (protocol, flags) in self.config.supported_protocols() {
            let initiate_connection_outcome = self
                .handle(Sv2ClientEvent::SetupConnectionTrigger(protocol, flags))
                .await;
            match initiate_connection_outcome {
                Ok(Sv2ClientOutcome::Ok) => {
                    debug!("Connection established with {:?}", protocol);
                }
                _ => {
                    return Err(Sv2ClientServiceError::FailedToInitiateConnection { protocol });
                }
            }

            let mut this = self.clone();
            tokio::spawn(async move {
                if let Err(e) = this.listen_for_messages_via_tcp(protocol).await {
                    error!("Error listening for messages: {:?}", e);
                }
            });
        }

        let mut this = self.clone();
        if let Some(_sibling_server_service_io) = this.sibling_server_service_io.clone() {
            tokio::spawn(async move {
                if let Err(e) = this.listen_for_events_via_sibling_server_service().await {
                    error!("Error listening for events: {:?}", e);
                }
            });
        }

        let mut this = self.clone();
        if let Some(event_injector) = this.event_injector.clone() {
            tokio::spawn(async move {
                if let Err(e) = this
                    .listen_for_events_via_event_injector(event_injector)
                    .await
                {
                    error!("Error listening for events: {:?}", e);
                }
            });
        }

        if !Self::has_null_handler(Protocol::MiningProtocol) {
            match self
                .handle(Sv2ClientEvent::MiningTrigger(MiningClientTrigger::Start))
                .await
            {
                Ok(_) => {
                    debug!("Mining handler started");
                }
                Err(e) => {
                    error!("Failed to start mining handler: {:?}", e);
                    return Err(Sv2ClientServiceError::FailedToStartMiningHandler);
                }
            }
        }

        // todo: start job declaration handler

        if !Self::has_null_handler(Protocol::TemplateDistributionProtocol) {
            match self
                .handle(Sv2ClientEvent::TemplateDistributionTrigger(
                    TemplateDistributionClientTrigger::Start,
                ))
                .await
            {
                Ok(_) => {
                    debug!("Template distribution handler started");
                }
                Err(e) => {
                    error!("Failed to start template distribution handler: {:?}", e);
                    return Err(Sv2ClientServiceError::FailedToStartTemplateDistributionHandler);
                }
            }
        }

        // wait for cancellation token to be cancelled
        self.cancellation_token.cancelled().await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::client::service::config::Sv2ClientServiceConfig;
    use crate::client::service::config::Sv2ClientServiceMiningConfig;
    use crate::client::service::config::Sv2ClientServiceTemplateDistributionConfig;
    use crate::client::service::event::Sv2ClientEvent;
    use crate::client::service::outcome::Sv2ClientOutcome;
    use crate::client::service::subprotocols::mining::handler::NullSv2MiningClientHandler;
    use crate::client::service::subprotocols::mining::handler::Sv2MiningClientHandler;
    use crate::client::service::subprotocols::template_distribution::handler::NullSv2TemplateDistributionClientHandler;
    use crate::client::service::subprotocols::template_distribution::handler::Sv2TemplateDistributionClientHandler;
    use crate::client::service::subprotocols::template_distribution::trigger::TemplateDistributionClientTrigger;
    use crate::client::service::Sv2ClientEventError;
    use crate::client::service::Sv2ClientService;
    use crate::server::service::config::Sv2ServerServiceConfig;
    use crate::server::service::config::Sv2ServerServiceMiningConfig;
    use crate::server::service::config::Sv2ServerTcpConfig;
    use crate::server::service::event::Sv2ServerEvent;
    use crate::server::service::event::Sv2ServerEventError;
    use crate::server::service::outcome::Sv2ServerOutcome;
    use crate::server::service::subprotocols::mining::handler::Sv2MiningServerHandler;
    use crate::server::service::subprotocols::mining::trigger::MiningServerTrigger;
    use crate::server::service::Sv2ServerService;
    use crate::Sv2Service;
    use integration_tests_sv2::interceptor::MessageDirection;
    use integration_tests_sv2::start_sniffer;
    use integration_tests_sv2::start_template_provider;
    use key_utils::Secp256k1PublicKey;
    use key_utils::Secp256k1SecretKey;
    use std::net::IpAddr;
    use std::net::Ipv4Addr;
    use std::net::SocketAddr;
    use std::str::FromStr;
    use stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::B064K;
    use stratum_common::roles_logic_sv2::common_messages_sv2::Protocol;
    use stratum_common::roles_logic_sv2::mining_sv2::OpenExtendedMiningChannel;
    use stratum_common::roles_logic_sv2::mining_sv2::OpenStandardMiningChannel;
    use stratum_common::roles_logic_sv2::mining_sv2::SetCustomMiningJob;
    use stratum_common::roles_logic_sv2::mining_sv2::SubmitSharesExtended;
    use stratum_common::roles_logic_sv2::mining_sv2::SubmitSharesStandard;
    use stratum_common::roles_logic_sv2::mining_sv2::UpdateChannel;
    use stratum_common::roles_logic_sv2::mining_sv2::{
        CloseChannel, NewExtendedMiningJob, NewMiningJob, OpenExtendedMiningChannelSuccess,
        OpenMiningChannelError, OpenStandardMiningChannelSuccess, SetCustomMiningJobError,
        SetCustomMiningJobSuccess, SetExtranoncePrefix, SetGroupChannel,
        SetNewPrevHash as SetNewPrevHashMining, SetTarget, SubmitSharesError, SubmitSharesSuccess,
        UpdateChannelError,
    };
    use stratum_common::roles_logic_sv2::template_distribution_sv2::{
        NewTemplate, RequestTransactionDataError, RequestTransactionDataSuccess, SetNewPrevHash,
    };
    use stratum_common::roles_logic_sv2::template_distribution_sv2::{
        MESSAGE_TYPE_COINBASE_OUTPUT_CONSTRAINTS, MESSAGE_TYPE_REQUEST_TRANSACTION_DATA,
    };
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    // A dummy mining handler that is not null, but not actually handling anything
    #[derive(Debug, Clone, Default)]
    struct DummyMiningClientHandler;

    impl Sv2MiningClientHandler for DummyMiningClientHandler {
        async fn start(&mut self) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_open_standard_mining_channel_success(
            &mut self,
            _open_standard_mining_channel_success: OpenStandardMiningChannelSuccess<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_open_extended_mining_channel_success(
            &mut self,
            _open_extended_mining_channel_success: OpenExtendedMiningChannelSuccess<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_open_mining_channel_error(
            &mut self,
            _open_mining_channel_error: OpenMiningChannelError<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_update_channel_error(
            &mut self,
            _update_channel_error: UpdateChannelError<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_close_channel(
            &mut self,
            _close_channel: CloseChannel<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_set_extranonce_prefix(
            &mut self,
            _set_extranonce_prefix: SetExtranoncePrefix<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_submit_shares_success(
            &mut self,
            _submit_shares_success: SubmitSharesSuccess,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_submit_shares_error(
            &mut self,
            _submit_shares_error: SubmitSharesError<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_new_mining_job(
            &mut self,
            _new_mining_job: NewMiningJob<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_new_extended_mining_job(
            &mut self,
            _new_extended_mining_job: NewExtendedMiningJob<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_set_new_prev_hash(
            &mut self,
            _set_new_prev_hash: SetNewPrevHashMining<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_set_custom_mining_job_success(
            &mut self,
            _set_custom_mining_job_success: SetCustomMiningJobSuccess,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_set_custom_mining_job_error(
            &mut self,
            _set_custom_mining_job_error: SetCustomMiningJobError<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_set_target(
            &mut self,
            _set_target: SetTarget<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_set_group_channel(
            &mut self,
            _set_group_channel: SetGroupChannel<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }
    }

    // A dummy template distribution handler that is not null, but not actually handling anything
    #[derive(Debug, Clone, Default)]
    struct DummyTemplateDistributionClientHandler;

    impl Sv2TemplateDistributionClientHandler for DummyTemplateDistributionClientHandler {
        async fn start(&mut self) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_new_template(
            &self,
            _template: NewTemplate<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_set_new_prev_hash(
            &self,
            _prev_hash: SetNewPrevHash<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_request_transaction_data_success(
            &self,
            _transaction_data: RequestTransactionDataSuccess<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_request_transaction_data_error(
            &self,
            _error: RequestTransactionDataError<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }
    }

    // A template distribution handler that is sending events using the Sibling IO feature
    #[derive(Debug, Clone, Default)]
    struct SiblingIoTemplateDistributionClientHandler;

    impl Sv2TemplateDistributionClientHandler for SiblingIoTemplateDistributionClientHandler {
        async fn start(&mut self) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_new_template(
            &self,
            template: NewTemplate<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            let outcome = Sv2ClientOutcome::TriggerNewEvent(Box::new(
                Sv2ClientEvent::SendEventToSiblingServerService(Box::new(
                    Sv2ServerEvent::MiningTrigger(MiningServerTrigger::NewTemplate(
                        template.into_static(),
                    )),
                )),
            ));
            Ok(outcome)
        }

        async fn handle_set_new_prev_hash(
            &self,
            prev_hash: SetNewPrevHash<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            let outcome = Sv2ClientOutcome::TriggerNewEvent(Box::new(
                Sv2ClientEvent::SendEventToSiblingServerService(Box::new(
                    Sv2ServerEvent::MiningTrigger(MiningServerTrigger::SetNewPrevHash(
                        prev_hash.into_static(),
                    )),
                )),
            ));
            Ok(outcome)
        }

        async fn handle_request_transaction_data_success(
            &self,
            _transaction_data: RequestTransactionDataSuccess<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }

        async fn handle_request_transaction_data_error(
            &self,
            _error: RequestTransactionDataError<'_>,
        ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
            Ok(Sv2ClientOutcome::Ok)
        }
    }

    // A Dummy Mining Server Handler that is not null, but not actually handling anything
    #[derive(Debug, Clone, Default)]
    struct DummyMiningServerHandler;
    impl Sv2MiningServerHandler for DummyMiningServerHandler {
        async fn start(&mut self) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
            Ok(Sv2ServerOutcome::Ok)
        }

        async fn on_new_template(
            &self,
            _m: NewTemplate<'static>,
        ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
            Ok(Sv2ServerOutcome::Ok)
        }

        async fn on_set_new_prev_hash(
            &self,
            _m: SetNewPrevHash<'static>,
        ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
            Ok(Sv2ServerOutcome::Ok)
        }

        fn setup_connection_success_flags(&self) -> u32 {
            0
        }

        async fn add_client(&mut self, _client_id: u32, _flags: u32) {}

        async fn remove_client(&mut self, _client_id: u32) {}

        async fn handle_open_standard_mining_channel(
            &self,
            _client_id: u32,
            _m: OpenStandardMiningChannel<'static>,
        ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
            Ok(Sv2ServerOutcome::Ok)
        }

        async fn handle_open_extended_mining_channel(
            &self,
            _client_id: u32,
            _m: OpenExtendedMiningChannel<'static>,
        ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
            Ok(Sv2ServerOutcome::Ok)
        }

        async fn handle_update_channel(
            &self,
            _client_id: u32,
            _m: UpdateChannel<'static>,
        ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
            Ok(Sv2ServerOutcome::Ok)
        }

        async fn handle_close_channel(
            &self,
            _client_id: u32,
            _m: CloseChannel<'static>,
        ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
            Ok(Sv2ServerOutcome::Ok)
        }

        async fn handle_submit_shares_standard(
            &self,
            _client_id: u32,
            _m: SubmitSharesStandard,
        ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
            Ok(Sv2ServerOutcome::Ok)
        }

        async fn handle_submit_shares_extended(
            &self,
            _client_id: u32,
            _m: SubmitSharesExtended<'static>,
        ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
            Ok(Sv2ServerOutcome::Ok)
        }

        async fn handle_set_custom_mining_job(
            &self,
            _client_id: u32,
            _m: SetCustomMiningJob<'static>,
        ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
            Ok(Sv2ServerOutcome::Ok)
        }
    }

    #[tokio::test]
    async fn sv2_client_service_initiate_connection_success() {
        // start a TemplateProvider
        let (_tp, tp_addr) = integration_tests_sv2::start_template_provider(
            None,
            integration_tests_sv2::template_provider::DifficultyLevel::Mid,
        );

        let template_distribution_config = Sv2ClientServiceTemplateDistributionConfig {
            coinbase_output_constraints: (1, 1),
            server_addr: tp_addr,
            auth_pk: None,
            setup_connection_flags: 0,
        };

        let sv2_client_service_config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: Some("localhost".to_string()),
            endpoint_port: Some(8080),
            vendor: Some("test".to_string()),
            hardware_version: Some("test".to_string()),
            firmware: Some("test".to_string()),
            device_id: Some("test".to_string()),
            mining_config: None,
            job_declaration_config: None,
            template_distribution_config: Some(template_distribution_config.clone()),
        };

        let template_distribution_handler = DummyTemplateDistributionClientHandler;

        let cancellation_token = CancellationToken::new();

        let mut sv2_client_service = Sv2ClientService::new(
            sv2_client_service_config,
            NullSv2MiningClientHandler,
            template_distribution_handler,
            cancellation_token,
        )
        .unwrap();

        assert!(
            !sv2_client_service
                .is_connected(Protocol::TemplateDistributionProtocol)
                .await
        );

        let initiate_connection_event = Sv2ClientEvent::SetupConnectionTrigger(
            Protocol::TemplateDistributionProtocol,
            template_distribution_config.setup_connection_flags,
        );

        let initiate_connection_outcome = sv2_client_service
            .handle(initiate_connection_event)
            .await
            .unwrap();

        // Verify the outcome matches what we expect
        assert!(matches!(initiate_connection_outcome, Sv2ClientOutcome::Ok));

        // Check connection state on the updated service instance
        assert!(
            sv2_client_service
                .is_connected(Protocol::TemplateDistributionProtocol)
                .await
        );
    }

    #[tokio::test]
    async fn sv2_client_service_initiate_connection_error() {
        // start a TemplateProvider
        let (_tp, tp_addr) = integration_tests_sv2::start_template_provider(
            None,
            integration_tests_sv2::template_provider::DifficultyLevel::Mid,
        );

        let sv2_client_service_config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: Some("localhost".to_string()),
            endpoint_port: Some(8080),
            vendor: Some("test".to_string()),
            hardware_version: Some("test".to_string()),
            firmware: Some("test".to_string()),
            device_id: Some("test".to_string()),
            mining_config: Some(Sv2ClientServiceMiningConfig {
                server_addr: tp_addr,
                auth_pk: None,
                setup_connection_flags: 0,
            }),
            job_declaration_config: None,
            template_distribution_config: None,
        };

        let mining_handler = DummyMiningClientHandler;

        let cancellation_token = CancellationToken::new();

        let mut sv2_client_service = Sv2ClientService::new(
            sv2_client_service_config,
            mining_handler,
            NullSv2TemplateDistributionClientHandler,
            cancellation_token,
        )
        .unwrap();

        assert!(
            !sv2_client_service
                .is_connected(Protocol::TemplateDistributionProtocol)
                .await
        );

        let initiate_connection_event =
            Sv2ClientEvent::SetupConnectionTrigger(Protocol::TemplateDistributionProtocol, 0);
        let _initiate_connection_outcome =
            sv2_client_service.handle(initiate_connection_event).await;

        // Verify we get a SetupConnectionError and check its contents
        // match initiate_connection_outcome {
        //     Err(Sv2ClientEventError::SetupConnectionError(error_code)) => {
        //         // The error should indicate that the protocol is unsupported
        //         assert!(error_code.contains("unsupported-protocol"),
        //             "Expected error about unsupported protocol, got: {}", error_code);
        //     }
        //     other => panic!("Expected SetupConnectionError, got: {:?}", other),
        // }

        // the assertion above is currently impossible, until the following is fixed:
        // https://github.com/Sjors/bitcoin/issues/84

        // Check connection state on the updated service instance - should still be disconnected
        assert!(
            !sv2_client_service
                .is_connected(Protocol::TemplateDistributionProtocol)
                .await
        );
    }

    #[tokio::test]
    async fn sv2_client_service_bad_config() {
        let config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: Some("localhost".to_string()),
            endpoint_port: Some(8080),
            vendor: Some("test".to_string()),
            hardware_version: Some("test".to_string()),
            firmware: Some("test".to_string()),
            device_id: Some("test".to_string()),
            mining_config: None,
            job_declaration_config: None,
            template_distribution_config: None,
        };

        // add a dummy template distribution handler to (which is not null)
        let template_distribution_handler = DummyTemplateDistributionClientHandler;

        let cancellation_token = CancellationToken::new();

        let result = Sv2ClientService::new(
            config,
            NullSv2MiningClientHandler,
            template_distribution_handler.clone(),
            cancellation_token,
        );

        // we expect an error, because the template distribution config is None
        assert!(result.is_err());

        let template_distribution_config = Sv2ClientServiceTemplateDistributionConfig {
            coinbase_output_constraints: (1, 1),
            server_addr: std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                8080,
            ),
            auth_pk: None,
            setup_connection_flags: 0,
        };

        // --------------------------------------------------------------------------------------------

        let config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: Some("localhost".to_string()),
            endpoint_port: Some(8080),
            vendor: Some("test".to_string()),
            hardware_version: Some("test".to_string()),
            firmware: Some("test".to_string()),
            device_id: Some("test".to_string()),
            mining_config: None,
            job_declaration_config: None,
            template_distribution_config: Some(template_distribution_config), // we are signaling that we support template distribution
        };

        // but now we are using a null template distribution handler, which is also not allowed
        let template_distribution_handler = NullSv2TemplateDistributionClientHandler;

        let cancellation_token = CancellationToken::new();

        let result = Sv2ClientService::new(
            config,
            NullSv2MiningClientHandler,
            template_distribution_handler,
            cancellation_token,
        );
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sv2_client_service_shutdown_when_not_connected() {
        let (_tp, tp_addr) = integration_tests_sv2::start_template_provider(
            None,
            integration_tests_sv2::template_provider::DifficultyLevel::Mid,
        );

        let template_distribution_config = Sv2ClientServiceTemplateDistributionConfig {
            coinbase_output_constraints: (1, 1),
            server_addr: tp_addr,
            auth_pk: None,
            setup_connection_flags: 0,
        };

        let sv2_client_service_config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: Some("localhost".to_string()),
            endpoint_port: Some(8080),
            vendor: Some("test".to_string()),
            hardware_version: Some("test".to_string()),
            firmware: Some("test".to_string()),
            device_id: Some("test".to_string()),
            mining_config: None,
            job_declaration_config: None,
            template_distribution_config: Some(template_distribution_config),
        };

        let template_distribution_handler = DummyTemplateDistributionClientHandler;

        let cancellation_token = CancellationToken::new();

        let sv2_client_service = Sv2ClientService::new(
            sv2_client_service_config,
            NullSv2MiningClientHandler,
            template_distribution_handler,
            cancellation_token.clone(),
        )
        .unwrap();

        // Verify initial state
        assert!(
            !sv2_client_service
                .is_connected(Protocol::TemplateDistributionProtocol)
                .await
        );

        // Shutdown should work even when not connected
        cancellation_token.cancel();

        // Verify final state
        assert!(
            !sv2_client_service
                .is_connected(Protocol::TemplateDistributionProtocol)
                .await
        );
    }

    #[tokio::test]
    async fn sv2_client_service_shutdown_when_connected() {
        let (_tp, tp_addr) = integration_tests_sv2::start_template_provider(
            None,
            integration_tests_sv2::template_provider::DifficultyLevel::Mid,
        );

        let mining_handler = NullSv2MiningClientHandler;

        let template_distribution_config = Sv2ClientServiceTemplateDistributionConfig {
            coinbase_output_constraints: (1, 1),
            server_addr: tp_addr,
            auth_pk: None,
            setup_connection_flags: 0,
        };

        let sv2_client_service_config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: Some("localhost".to_string()),
            endpoint_port: Some(8080),
            vendor: Some("test".to_string()),
            hardware_version: Some("test".to_string()),
            firmware: Some("test".to_string()),
            device_id: Some("test".to_string()),
            mining_config: None,
            job_declaration_config: None,
            template_distribution_config: Some(template_distribution_config.clone()),
        };

        let template_distribution_handler = DummyTemplateDistributionClientHandler;

        let cancellation_token = CancellationToken::new();

        let mut sv2_client_service = Sv2ClientService::new(
            sv2_client_service_config,
            mining_handler,
            template_distribution_handler,
            cancellation_token.clone(),
        )
        .unwrap();

        // Connect to the server
        let outcome = sv2_client_service
            .handle(Sv2ClientEvent::SetupConnectionTrigger(
                Protocol::TemplateDistributionProtocol,
                template_distribution_config.setup_connection_flags,
            ))
            .await
            .unwrap();
        assert!(matches!(outcome, Sv2ClientOutcome::Ok));

        // Start message listener in background
        let mut sv2_client_service_clone = sv2_client_service.clone();
        let (tx, mut rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let result = sv2_client_service_clone
                .listen_for_messages_via_tcp(Protocol::TemplateDistributionProtocol)
                .await;
            tx.send(()).await.unwrap();
            result
        });

        // Verify connected state
        assert!(
            sv2_client_service
                .is_connected(Protocol::TemplateDistributionProtocol)
                .await
        );

        // Shutdown the client
        cancellation_token.cancel();

        // Wait for listener to exit
        rx.recv().await.unwrap();

        // Verify final state
        assert!(
            !sv2_client_service
                .is_connected(Protocol::TemplateDistributionProtocol)
                .await
        );
    }

    #[tokio::test]
    async fn test_event_injector() {
        let (_tp, tp_addr) = integration_tests_sv2::start_template_provider(
            None,
            integration_tests_sv2::template_provider::DifficultyLevel::Mid,
        );

        // Start a sniffer to intercept messages between the client and the Template Provider.
        let (tp_sniffer, tp_sniffer_addr) = start_sniffer("", tp_addr, false, vec![], None);

        // Allow some time for the sniffer to initialize.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let template_distribution_config = Sv2ClientServiceTemplateDistributionConfig {
            coinbase_output_constraints: (1, 1),
            server_addr: tp_sniffer_addr,
            auth_pk: None,
            setup_connection_flags: 0,
        };

        let sv2_client_service_config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: Some("localhost".to_string()),
            endpoint_port: Some(8080),
            vendor: Some("test".to_string()),
            hardware_version: Some("test".to_string()),
            firmware: Some("test".to_string()),
            device_id: Some("test".to_string()),
            mining_config: None,
            job_declaration_config: None,
            template_distribution_config: Some(template_distribution_config),
        };

        let (tx, rx) = async_channel::unbounded::<Sv2ClientEvent<'static>>();

        let cancellation_token = CancellationToken::new();

        let sv2_client_service = Sv2ClientService::new_with_event_injector(
            sv2_client_service_config,
            NullSv2MiningClientHandler,
            DummyTemplateDistributionClientHandler,
            rx,
            cancellation_token.clone(),
        )
        .unwrap();

        let mut sv2_client_service_clone = sv2_client_service.clone();
        tokio::spawn(async move {
            sv2_client_service_clone.start().await.unwrap();
        });

        // Wait for client to be ready
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        assert!(
            sv2_client_service
                .is_connected(Protocol::TemplateDistributionProtocol)
                .await
        );

        // Send a event to the client service via the event injector.
        tx.send(Sv2ClientEvent::TemplateDistributionTrigger(
            TemplateDistributionClientTrigger::TransactionDataNeeded(0),
        ))
        .await
        .unwrap();

        // Wait for the sniffer to detect the transaction data needed message.
        tp_sniffer
            .wait_for_message_type(
                MessageDirection::ToUpstream,
                MESSAGE_TYPE_REQUEST_TRANSACTION_DATA,
            )
            .await;

        // Shutdown the client
        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn test_sibling_io() {
        let mut server_config = Sv2ServerServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            inactivity_limit: 3600,
            tcp_config: Sv2ServerTcpConfig {
                listen_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3333),
                pub_key: Secp256k1PublicKey::from_str(
                    "9auqWEzQDVyd2oe1JVGFLMLHZtCo2FFqZwtKA5gd9xbuEu7PH72",
                )
                .unwrap(),
                priv_key: Secp256k1SecretKey::from_str(
                    "mkDLTBBRxdBv998612qipDYoTK3YUrqLe8uWw7gu3iXbSrn2n",
                )
                .unwrap(),
                cert_validity: 3600,
            },
            mining_config: Some(Sv2ServerServiceMiningConfig {
                supported_flags: 0b0101,
            }),
            job_declaration_config: None,
            template_distribution_config: None,
        };

        let mut client_config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: None,
            endpoint_port: None,
            vendor: None,
            hardware_version: None,
            device_id: None,
            firmware: None,
            mining_config: None,
            job_declaration_config: None,
            template_distribution_config: Some(Sv2ClientServiceTemplateDistributionConfig {
                server_addr: SocketAddr::from_str("127.0.0.1:8442").unwrap(),
                auth_pk: None,
                coinbase_output_constraints: (1, 1),
                setup_connection_flags: 0,
            }),
        };

        // Start a Template Provider that simulates a Bitcoin node with SV2 support.
        let (_tp, tp_address) = start_template_provider(
            None,
            integration_tests_sv2::template_provider::DifficultyLevel::Mid,
        );

        // Start a sniffer to intercept messages between the client and the Template Provider.
        let (tp_sniffer, tp_sniffer_addr) = start_sniffer("", tp_address, false, vec![], None);

        // Update the client configuration to use the sniffer's address.
        if let Some(ref mut tdc) = client_config.template_distribution_config {
            tdc.server_addr = tp_sniffer_addr;
        }

        server_config.tcp_config.listen_address = SocketAddr::from_str("127.0.0.1:0").unwrap();

        // Allow some time for the sniffer to initialize.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Clone the template_distribution_config for later use to avoid moving it.
        let template_distribution_config = client_config
            .template_distribution_config
            .as_ref()
            .unwrap()
            .clone();

        // Initialize the handlers for TemplateDistribution and MiningServer.
        let tdc_handler = SiblingIoTemplateDistributionClientHandler;
        let mining_server_handler = DummyMiningServerHandler;

        let cancellation_token = CancellationToken::new();

        // Create the Sv2ServerService and its sibling IO for communication with the client.
        let (
            server_service,
            sibling_server_io, // SiblingIO is created here.
        ) = Sv2ServerService::new_with_sibling_io(
            server_config,
            mining_server_handler,
            cancellation_token.clone(),
        )
        .unwrap();

        // Create the Sv2ClientService and connect it to the server using the sibling IO.

        let mut client_service = Sv2ClientService::new_from_sibling_io(
            client_config.clone(),
            NullSv2MiningClientHandler,
            tdc_handler,
            sibling_server_io,
            cancellation_token.clone(),
        )
        .unwrap();

        let mut server_service_clone = server_service.clone();
        tokio::spawn(async move {
            server_service_clone.start().await.unwrap();
        });

        // Wait for server to be ready
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let mut client_service_clone = client_service.clone();
        tokio::spawn(async move {
            client_service_clone.start().await.unwrap();
        });

        // Wait for client to be ready
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Trigger the Template Provider to set coinbase output constraints.
        client_service
            .handle(Sv2ClientEvent::TemplateDistributionTrigger(
                TemplateDistributionClientTrigger::SetCoinbaseOutputConstraints(
                    template_distribution_config.coinbase_output_constraints.0,
                    template_distribution_config.coinbase_output_constraints.1,
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
            coinbase_prefix: stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::B0255::Owned(
                hex::decode("00").unwrap().to_vec(),
            ),
            coinbase_tx_input_sequence: 0,
            coinbase_tx_value_remaining: 0,
            coinbase_tx_outputs_count: 0,
            coinbase_tx_outputs:
                stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::B064K::Owned(
                    hex::decode("00").unwrap().to_vec(),
                ),
            coinbase_tx_locktime: 0,
            merkle_path: stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::Seq0255::new(
                Vec::new(),
            )
            .unwrap(),
        };

        // Send the NewTemplate message to the sibling server and verify the outcome.
        let new_template_outcome = client_service
            .handle(Sv2ClientEvent::SendEventToSiblingServerService(Box::new(
                Sv2ServerEvent::MiningTrigger(MiningServerTrigger::NewTemplate(new_template)),
            )))
            .await;

        // Assert that the outcome was sent to the sibling server.
        assert!(matches!(new_template_outcome, Ok(Sv2ClientOutcome::Ok)));

        // Create a dummy SetNewPrevHash message to simulate a new previous hash.
        let new_prev_hash = SetNewPrevHash {
            template_id: 0,
            prev_hash: stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::U256::Owned(
                hex::decode("00").unwrap().to_vec(),
            ),
            header_timestamp: 0,
            n_bits: 0,
            target: stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::U256::Owned(
                hex::decode("00").unwrap().to_vec(),
            ),
        };

        // Send the SetNewPrevHash message to the sibling server and verify the outcome.
        let new_prev_hash_outcome = client_service
            .handle(Sv2ClientEvent::SendEventToSiblingServerService(Box::new(
                Sv2ServerEvent::MiningTrigger(MiningServerTrigger::SetNewPrevHash(new_prev_hash)),
            )))
            .await
            .unwrap();

        // Assert that the outcome from the MiningServerHandler is received.
        assert!(matches!(new_prev_hash_outcome, Sv2ClientOutcome::Ok));

        // Shutdown the server and client services gracefully.
        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn test_sibling_io_with_wrong_constructor() {
        let mut server_config = Sv2ServerServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            inactivity_limit: 3600,
            tcp_config: Sv2ServerTcpConfig {
                listen_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3333),
                pub_key: Secp256k1PublicKey::from_str(
                    "9auqWEzQDVyd2oe1JVGFLMLHZtCo2FFqZwtKA5gd9xbuEu7PH72",
                )
                .unwrap(),
                priv_key: Secp256k1SecretKey::from_str(
                    "mkDLTBBRxdBv998612qipDYoTK3YUrqLe8uWw7gu3iXbSrn2n",
                )
                .unwrap(),
                cert_validity: 3600,
            },
            mining_config: Some(Sv2ServerServiceMiningConfig {
                supported_flags: 0b0101,
            }),
            job_declaration_config: None,
            template_distribution_config: None,
        };

        let mut client_config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: None,
            endpoint_port: None,
            vendor: None,
            hardware_version: None,
            device_id: None,
            firmware: None,
            mining_config: None,
            job_declaration_config: None,
            template_distribution_config: Some(Sv2ClientServiceTemplateDistributionConfig {
                server_addr: SocketAddr::from_str("127.0.0.1:8442").unwrap(),
                auth_pk: None,
                coinbase_output_constraints: (1, 1),
                setup_connection_flags: 0,
            }),
        };

        // Start a Template Provider that simulates a Bitcoin node with SV2 support.
        let (_tp, tp_address) = start_template_provider(
            None,
            integration_tests_sv2::template_provider::DifficultyLevel::Mid,
        );

        // Update the client configuration to use the sniffer's address.
        client_config
            .template_distribution_config
            .as_mut()
            .unwrap()
            .server_addr = tp_address;

        server_config.tcp_config.listen_address = SocketAddr::from_str("127.0.0.1:0").unwrap();

        // Allow some time for the sniffer to initialize.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Initialize the handlers for TemplateDistribution and MiningServer.
        let tdc_handler = SiblingIoTemplateDistributionClientHandler;
        let mining_handler = DummyMiningServerHandler;

        let cancellation_token = CancellationToken::new();

        // Create the Sv2ServerService with a wrong constructor.
        let server_service =
            Sv2ServerService::new(server_config, mining_handler, cancellation_token.clone())
                .unwrap();
        // Create the Sv2ClientService with a wrong constructor.
        let mut client_service = Sv2ClientService::new(
            client_config,
            NullSv2MiningClientHandler,
            tdc_handler,
            cancellation_token.clone(),
        )
        .unwrap();

        let mut server_service_clone = server_service.clone();
        tokio::spawn(async move {
            server_service_clone.start().await.unwrap();
        });

        // Wait for client and server to be ready
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let mut client_service_clone = client_service.clone();
        tokio::spawn(async move {
            client_service_clone.start().await.unwrap();
        });

        // Wait for client to be ready
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Create a dummy NewTemplate message to simulate a new mining template.
        let new_template = NewTemplate {
            template_id: 0,
            future_template: false,
            version: 0,
            coinbase_tx_version: 0,
            coinbase_prefix: stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::B0255::Owned(
                hex::decode("00").unwrap().to_vec(),
            ),
            coinbase_tx_input_sequence: 0,
            coinbase_tx_value_remaining: 0,
            coinbase_tx_outputs_count: 0,
            coinbase_tx_outputs:
                stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::B064K::Owned(
                    hex::decode("00").unwrap().to_vec(),
                ),
            coinbase_tx_locktime: 0,
            merkle_path: stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::Seq0255::new(
                Vec::new(),
            )
            .unwrap(),
        };

        // Send the NewTemplate message to the sibling server and verify the outcome.
        let new_template_outcome = client_service
            .handle(Sv2ClientEvent::SendEventToSiblingServerService(Box::new(
                Sv2ServerEvent::MiningTrigger(MiningServerTrigger::NewTemplate(new_template)),
            )))
            .await;

        // Assert that the error is of type Sv2ClientEventError::NoSiblingServerServiceIo.
        assert!(matches!(
            new_template_outcome,
            Err(Sv2ClientEventError::NoSiblingServerServiceIo)
        ));

        // Shutdown the server and client services gracefully.
        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn sv2_client_service_submit_solution() {
        use crate::client::service::event::Sv2ClientEvent;
        use crate::client::service::outcome::Sv2ClientOutcome;
        use crate::client::service::subprotocols::template_distribution::trigger::TemplateDistributionClientTrigger;
        use stratum_common::roles_logic_sv2::common_messages_sv2::Protocol;
        use stratum_common::roles_logic_sv2::template_distribution_sv2::SubmitSolution;

        // Start a TemplateProvider
        let (_tp, tp_addr) = integration_tests_sv2::start_template_provider(
            None,
            integration_tests_sv2::template_provider::DifficultyLevel::Mid,
        );

        let template_distribution_config = Sv2ClientServiceTemplateDistributionConfig {
            coinbase_output_constraints: (1, 1),
            server_addr: tp_addr,
            auth_pk: None,
            setup_connection_flags: 0,
        };

        let sv2_client_service_config = Sv2ClientServiceConfig {
            min_supported_version: 2,
            max_supported_version: 2,
            endpoint_host: Some("localhost".to_string()),
            endpoint_port: Some(8080),
            vendor: Some("test".to_string()),
            hardware_version: Some("test".to_string()),
            firmware: Some("test".to_string()),
            device_id: Some("test".to_string()),
            mining_config: None,
            job_declaration_config: None,
            template_distribution_config: Some(template_distribution_config.clone()),
        };

        let template_distribution_handler = DummyTemplateDistributionClientHandler;

        let cancellation_token = CancellationToken::new();

        let mut sv2_client_service = Sv2ClientService::new(
            sv2_client_service_config,
            NullSv2MiningClientHandler,
            template_distribution_handler,
            cancellation_token,
        )
        .unwrap();

        // Connect to the server
        let initiate_connection_event = Sv2ClientEvent::SetupConnectionTrigger(
            Protocol::TemplateDistributionProtocol,
            template_distribution_config.setup_connection_flags,
        );
        let outcome = sv2_client_service.handle(initiate_connection_event).await;
        assert!(matches!(outcome, Ok(Sv2ClientOutcome::Ok)));

        // Construct a dummy SubmitSolution message
        let submit_solution = SubmitSolution {
            template_id: 0,
            version: 0,
            header_timestamp: 0,
            header_nonce: 0,
            coinbase_tx: B064K::Owned(vec![]),
        };

        let submit_solution_event = Sv2ClientEvent::TemplateDistributionTrigger(
            TemplateDistributionClientTrigger::SubmitSolution(submit_solution),
        );

        let outcome = sv2_client_service.handle(submit_solution_event).await;
        assert!(matches!(outcome, Ok(Sv2ClientOutcome::Ok)));
    }
}
