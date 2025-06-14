use roles_logic_sv2::parsers::AnyMessage;

use crate::server::service::request::RequestToSv2Server;
/// An ordered sequence of Sv2 messages
/// to be delivered to a specific client.
#[derive(Debug, Clone)]
pub struct Sv2MessagesToClient<'a> {
    pub client_id: u32,
    pub messages: Vec<(AnyMessage<'a>, u8)>,
}

/// The Response type for the tower service [`crate::server::service::Sv2ServerService`].
#[derive(Debug, Clone)]
pub enum ResponseFromSv2Server<'a> {
    SendMessagesToClients(Box<Vec<Sv2MessagesToClient<'a>>>),
    TriggerNewRequest(Box<RequestToSv2Server<'a>>),
    Ok,
}
