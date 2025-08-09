use crate::server::service::event::Sv2ServerEvent;

/// The outcome type for [`crate::server::service::Sv2ServerService`].
#[derive(Debug, Clone)]
pub enum Sv2ServerOutcome<'a> {
    TriggerNewEvent(Box<Sv2ServerEvent<'a>>),
    Ok,
}
