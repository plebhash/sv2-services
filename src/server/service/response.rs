use crate::server::service::request::RequestToSv2Server;

/// The Response type for the tower service [`crate::server::service::Sv2ServerService`].
#[derive(Debug, Clone)]
pub enum ResponseFromSv2Server<'a> {
    TriggerNewRequest(Box<RequestToSv2Server<'a>>),
    Ok,
}
