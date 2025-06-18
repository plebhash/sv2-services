use roles_logic_sv2::template_distribution_sv2::{NewTemplate, SetNewPrevHash};

/// Requests to the Server Service that are specific to the Mining subprotocol.
#[derive(Debug, Clone)]
pub enum MiningServerTrigger<'a> {
    Start,
    NewTemplate(NewTemplate<'a>),
    SetNewPrevHash(SetNewPrevHash<'a>),
}
