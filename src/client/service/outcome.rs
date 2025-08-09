use crate::client::service::event::Sv2ClientEvent;

/// The outcome type for the service [`crate::client::service::Sv2ClientService`].
#[derive(Debug)]
pub enum Sv2ClientOutcome<'a> {
    TriggerNewEvent(Box<Sv2ClientEvent<'a>>),
    Ok,
}
