use crate::client::service::subprotocols::mining::trigger::MiningClientTrigger;
use crate::client::service::subprotocols::template_distribution::trigger::TemplateDistributionClientTrigger;
use crate::server::service::event::Sv2ServerEvent;
use crate::Sv2MessageIoError;
use stratum_common::roles_logic_sv2::common_messages_sv2::Protocol;
use stratum_common::roles_logic_sv2::parsers::{AnyMessage, Mining, TemplateDistribution};

/// The event type for the [`crate::client::service::Sv2ClientService`] service.
#[derive(Debug, Clone)]
pub enum Sv2ClientEvent<'a> {
    /// Trigger for the client to initiate a connection to the server under some subprotocol.
    SetupConnectionTrigger(Protocol, u32), // protocol, flags
    /// Some Sv2 message addressed to the client.
    /// Could belong to any subprotocol.
    IncomingMessage(AnyMessage<'a>),
    MiningTrigger(MiningClientTrigger),
    TemplateDistributionTrigger(TemplateDistributionClientTrigger<'a>),
    SendEventToSiblingServerService(Box<Sv2ServerEvent<'a>>),
    SendMessageToMiningServer(Box<Mining<'a>>),
    SendMessageToTemplateDistributionServer(Box<TemplateDistribution<'a>>),
    // SendMessageToJobDeclarationServer(Box<(JobDeclaration<'a>, u8)>),
    /// Execute an ordered sequence of events.
    MultipleEvents(Box<Vec<Sv2ClientEvent<'a>>>),
}

/// The error type for the [`crate::client::service::Sv2ClientService`] service.
#[derive(Debug, Clone)]
pub enum Sv2ClientEventError {
    BadRouting,
    UnsupportedMessage,
    UnsupportedProtocol { protocol: Protocol },
    IsNotConnected,
    SetupConnectionError(String),
    ConnectionError(String),
    StringConversionError(String),
    NoSiblingServerServiceIo,
    FailedToSendEventToSiblingServerService,
    U256ConversionError(String),
    MiningHandlerError(String),
    TemplateDistributionHandlerError(String),
    JobDeclarationHandlerError(String),
}

impl From<Sv2MessageIoError> for Sv2ClientEventError {
    fn from(error: Sv2MessageIoError) -> Self {
        match error {
            Sv2MessageIoError::SendError => {
                Sv2ClientEventError::ConnectionError("Failed to send message".to_string())
            }
            Sv2MessageIoError::FrameError => {
                Sv2ClientEventError::ConnectionError("Failed to create frame".to_string())
            }
            Sv2MessageIoError::RecvError => {
                Sv2ClientEventError::ConnectionError("Failed to receive message".to_string())
            }
        }
    }
}
