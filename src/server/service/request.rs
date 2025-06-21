use stratum_common::roles_logic_sv2::common_messages_sv2::Protocol;
use stratum_common::roles_logic_sv2::parsers::AnyMessage;

use crate::client::service::request::RequestToSv2Client;
use crate::server::service::client::Sv2MessagesToClient;
use crate::server::service::subprotocols::mining::trigger::MiningServerTrigger;

/// The request type for the [`crate::server::service::Sv2ServerService`] service.
#[derive(Debug, Clone)]
pub enum RequestToSv2Server<'a> {
    /// Some Sv2 message addressed to the server.
    /// Could belong to any subprotocol.
    IncomingMessage(Sv2MessageToServer<'a>),
    /// Some trigger for the mining subprotocol service
    MiningTrigger(MiningServerTrigger<'a>),
    // todo:
    // JobDeclarationTrigger(RequestToSv2JobDeclarationServer<'a>),
    // TemplateDistributionTrigger(RequestToSv2TemplateDistributionServer<'a>),
    /// The request is boxed to break the recursive type definition between RequestToSv2Client and RequestToSv2Server.
    SendRequestToSiblingClientService(Box<RequestToSv2Client<'a>>),
    /// Send ordered sequence of Sv2 messages to a specific client.
    SendMessagesToClient(Box<Sv2MessagesToClient<'a>>),
    /// Send ordered sequences of Sv2 messages to different clients.
    SendMessagesToClients(Box<Vec<Sv2MessagesToClient<'a>>>),
    /// Execute an ordered sequence of requests.
    MultipleRequests(Box<Vec<RequestToSv2Server<'a>>>),
}

/// A Sv2 message addressed to the server, to be used as the request type of [`crate::server::service::Sv2ServerService`].
///
/// The client_id is always Some(id), with the exception of the initial SetupConnection request,
/// where the client_id is allocated by the server and returned in the response.
#[derive(Debug, Clone)]
pub struct Sv2MessageToServer<'a> {
    pub client_id: Option<u32>,
    pub message: AnyMessage<'a>,
}

/// The error type for the [`crate::server::service::Sv2ServerService`] service.
#[derive(Debug, Clone)]
pub enum RequestToSv2ServerError {
    IdNotFound,
    IdMustBeSome,
    BadRouting,
    UnsupportedMessage,
    FailedToSendResponseToClient,
    UnsupportedProtocol { protocol: Protocol },
    FailedToSendRequestToSiblingClientService,
    NoSiblingClientService,
    MiningHandlerError(String),
    TemplateDistributionHandlerError(String),
    JobDeclarationHandlerError(String),
}
