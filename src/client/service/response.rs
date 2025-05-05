use crate::client::service::request::RequestToSv2Client;
use crate::client::service::subprotocols::mining::response::ResponseToMiningTrigger;
use crate::client::service::subprotocols::template_distribution::response::ResponseToTemplateDistributionTrigger;
use crate::server::service::request::RequestToSv2Server;
use roles_logic_sv2::parsers::AnyMessage;

/// The response type for the tower service [`crate::client::service::Sv2ClientService`].
#[derive(Debug)]
pub enum ResponseFromSv2Client<'a> {
    SendToServer(AnyMessage<'a>),
    ResponseToTemplateDistributionTrigger(ResponseToTemplateDistributionTrigger),
    ResponseToMiningTrigger(ResponseToMiningTrigger),
    TriggerNewRequest(RequestToSv2Client<'a>),
    SentRequestToSiblingServerService(RequestToSv2Server<'a>),
    Ok,
    ToDo, // dummy placeholder for future response types (e.g.: Relay)
}
