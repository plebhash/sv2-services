use crate::client::service::request::RequestToSv2Client;

/// The response type for the tower service [`crate::client::service::Sv2ClientService`].
#[derive(Debug)]
pub enum ResponseFromSv2Client<'a> {
    TriggerNewRequest(Box<RequestToSv2Client<'a>>),
    Ok,
}
