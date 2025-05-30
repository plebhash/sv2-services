use crate::client::service::request::RequestToSv2Client;
use roles_logic_sv2::parsers::AnyMessage;

/// The response type for the tower service [`crate::client::service::Sv2ClientService`].
#[derive(Debug)]
pub enum ResponseFromSv2Client<'a> {
    SendToServer(Box<AnyMessage<'a>>),
    TriggerNewRequest(Box<RequestToSv2Client<'a>>),
    Ok,
}
