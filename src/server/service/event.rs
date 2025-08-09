use stratum_common::roles_logic_sv2::common_messages_sv2::Protocol;
use stratum_common::roles_logic_sv2::parsers::AnyMessage;

use crate::client::service::event::Sv2ClientEvent;
use crate::server::service::client::Sv2MessagesToClient;
use crate::server::service::subprotocols::mining::trigger::MiningServerTrigger;

/// The event type for [`crate::server::service::Sv2ServerService`].
#[derive(Debug, Clone)]
pub enum Sv2ServerEvent<'a> {
    /// Some Sv2 message addressed to the server.
    /// Could belong to any subprotocol.
    IncomingMessage(Sv2MessageToServer<'a>),
    /// Some trigger for the mining subprotocol service
    MiningTrigger(MiningServerTrigger<'a>),
    // todo:
    // JobDeclarationTrigger(JobDeclarationTrigger<'a>),
    // TemplateDistributionTrigger(TemplateDistributionTrigger<'a>),
    /// The event is boxed to break the recursive type definition between Sv2ClientEvent and Sv2ServerEvent.
    SendEventToSiblingClientService(Box<Sv2ClientEvent<'a>>),
    /// Send ordered sequence of Sv2 messages to a specific client.
    SendMessagesToClient(Box<Sv2MessagesToClient<'a>>),
    /// Send ordered sequences of Sv2 messages to different clients.
    SendMessagesToClients(Box<Vec<Sv2MessagesToClient<'a>>>),
    /// Execute an ordered sequence of events.
    MultipleEvents(Box<Vec<Sv2ServerEvent<'a>>>),
}

/// A Sv2 message addressed to the server, to be used as the event type of [`crate::server::service::Sv2ServerService`].
///
/// The client_id is always Some(id), with the exception of the initial SetupConnection event,
/// where the client_id is allocated by the server and returned in the outcome.
#[derive(Debug, Clone)]
pub struct Sv2MessageToServer<'a> {
    pub client_id: Option<u32>,
    pub message: AnyMessage<'a>,
}

/// The error type for [`crate::server::service::Sv2ServerService`].
#[derive(Debug, Clone)]
pub enum Sv2ServerEventError {
    IdNotFound,
    IdMustBeSome,
    BadRouting,
    UnsupportedMessage,
    FailedToSendOutcome,
    UnsupportedProtocol { protocol: Protocol },
    FailedToSendEventToSiblingClientService,
    FailedToSendMessageToClient,
    NoSiblingClientService,
    MiningHandlerError(String),
    TemplateDistributionHandlerError(String),
    JobDeclarationHandlerError(String),
}
