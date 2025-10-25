use crate::client::service::event::{Sv2ClientEvent, Sv2ClientEventError};
use crate::client::service::outcome::Sv2ClientOutcome;

use stratum_common::roles_logic_sv2::parsers_sv2::TemplateDistribution;
use stratum_common::roles_logic_sv2::template_distribution_sv2::{
    CoinbaseOutputConstraints, NewTemplate, RequestTransactionData, RequestTransactionDataError,
    RequestTransactionDataSuccess, SetNewPrevHash, SubmitSolution,
};

/// Trait that must be implemented in case [`crate::client::service::Sv2ClientService`] supports the Template Distribution protocol
pub trait Sv2TemplateDistributionClientHandler {
    fn start(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Sv2ClientOutcome<'static>, Sv2ClientEventError>> + Send;

    fn handle_new_template(
        &self,
        template: NewTemplate<'static>,
    ) -> impl std::future::Future<Output = Result<Sv2ClientOutcome<'static>, Sv2ClientEventError>> + Send;

    fn handle_set_new_prev_hash(
        &self,
        prev_hash: SetNewPrevHash<'static>,
    ) -> impl std::future::Future<Output = Result<Sv2ClientOutcome<'static>, Sv2ClientEventError>> + Send;

    fn handle_request_transaction_data_success(
        &self,
        transaction_data: RequestTransactionDataSuccess<'static>,
    ) -> impl std::future::Future<Output = Result<Sv2ClientOutcome<'static>, Sv2ClientEventError>> + Send;

    fn handle_request_transaction_data_error(
        &self,
        error: RequestTransactionDataError<'static>,
    ) -> impl std::future::Future<Output = Result<Sv2ClientOutcome<'static>, Sv2ClientEventError>> + Send;

    fn transaction_data_needed(
        &self,
        template_id: u64,
    ) -> impl std::future::Future<Output = Result<Sv2ClientOutcome<'static>, Sv2ClientEventError>> + Send
    {
        let message =
            TemplateDistribution::RequestTransactionData(RequestTransactionData { template_id });

        async move {
            Ok(Sv2ClientOutcome::TriggerNewEvent(Box::new(
                Sv2ClientEvent::SendMessageToTemplateDistributionServer(Box::new(message)),
            )))
        }
    }

    fn set_coinbase_output_constraints(
        &self,
        coinbase_output_max_additional_size: u32,
        coinbase_output_max_additional_sigops: u16,
    ) -> impl std::future::Future<Output = Result<Sv2ClientOutcome<'static>, Sv2ClientEventError>> + Send
    {
        let message = TemplateDistribution::CoinbaseOutputConstraints(CoinbaseOutputConstraints {
            coinbase_output_max_additional_size,
            coinbase_output_max_additional_sigops,
        });

        async move {
            Ok(Sv2ClientOutcome::TriggerNewEvent(Box::new(
                Sv2ClientEvent::SendMessageToTemplateDistributionServer(Box::new(message)),
            )))
        }
    }

    fn submit_solution(
        &self,
        solution: SubmitSolution<'static>,
    ) -> impl std::future::Future<Output = Result<Sv2ClientOutcome<'static>, Sv2ClientEventError>> + Send
    {
        let message = TemplateDistribution::SubmitSolution(solution);

        async move {
            Ok(Sv2ClientOutcome::TriggerNewEvent(Box::new(
                Sv2ClientEvent::SendMessageToTemplateDistributionServer(Box::new(message)),
            )))
        }
    }
}

// -------------------------------------------------------------------------------------------------
// NullSv2TemplateDistributionClientHandler
// -------------------------------------------------------------------------------------------------

/// A [`Sv2TemplateDistributionClientHandler`] implementation that does nothing.
///
/// It should be used when creating a [`crate::client::service::Sv2ClientService`] that
/// does not support the Template Distribution protocol.
#[derive(Debug, Clone)]
pub struct NullSv2TemplateDistributionClientHandler;

impl Sv2TemplateDistributionClientHandler for NullSv2TemplateDistributionClientHandler {
    async fn start(&mut self) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        unimplemented!("NullSv2TemplateDistributionClientHandler does not implement start");
    }

    async fn handle_new_template(
        &self,
        _template: NewTemplate<'static>,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        unimplemented!(
            "NullSv2TemplateDistributionClientHandler does not implement handle_new_template"
        );
    }

    async fn handle_set_new_prev_hash(
        &self,
        _prev_hash: SetNewPrevHash<'static>,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        unimplemented!(
            "NullSv2TemplateDistributionClientHandler does not implement handle_set_new_prev_hash"
        );
    }

    async fn handle_request_transaction_data_success(
        &self,
        _transaction_data: RequestTransactionDataSuccess<'static>,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        unimplemented!("NullSv2TemplateDistributionClientHandler does not implement handle_request_transaction_data_success");
    }

    async fn handle_request_transaction_data_error(
        &self,
        _error: RequestTransactionDataError<'static>,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        unimplemented!("NullSv2TemplateDistributionClientHandler does not implement handle_request_transaction_data_error");
    }

    async fn transaction_data_needed(
        &self,
        _template_id: u64,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        unimplemented!(
            "NullSv2TemplateDistributionClientHandler does not implement request_transaction_data"
        );
    }

    async fn set_coinbase_output_constraints(
        &self,
        _coinbase_output_max_additional_size: u32,
        _coinbase_output_max_additional_sigops: u16,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        unimplemented!("NullSv2TemplateDistributionClientHandler does not implement send_coinbase_output_constraints");
    }
}
