use roles_logic_sv2::template_distribution_sv2::SubmitSolution;

/// Requests to the Client Service that are specific to the Template Distribution protocol
#[derive(Debug, Clone)]
pub enum TemplateDistributionClientTrigger<'a> {
    SetCoinbaseOutputConstraints(u32, u16),
    TransactionDataNeeded(u64),
    SubmitSolution(SubmitSolution<'a>),
}
