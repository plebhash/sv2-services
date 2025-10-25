use anyhow::Result;
use sv2_services::client::service::event::Sv2ClientEvent;
use sv2_services::client::service::event::Sv2ClientEventError;
use sv2_services::client::service::outcome::Sv2ClientOutcome;
use sv2_services::client::service::subprotocols::template_distribution::handler::Sv2TemplateDistributionClientHandler;
use sv2_services::client::service::subprotocols::template_distribution::trigger::TemplateDistributionClientTrigger;
use sv2_services::roles_logic_sv2::template_distribution_sv2::{
    NewTemplate, RequestTransactionDataError, RequestTransactionDataSuccess, SetNewPrevHash,
};
use tracing::info;
#[derive(Debug, Clone, Default)]
pub struct MyTemplateDistributionHandler {
    coinbase_output_max_additional_size: u32,
    coinbase_output_max_additional_sigops: u16,
}

impl MyTemplateDistributionHandler {
    pub fn new(
        coinbase_output_max_additional_size: u32,
        coinbase_output_max_additional_sigops: u16,
    ) -> Self {
        Self {
            coinbase_output_max_additional_size,
            coinbase_output_max_additional_sigops,
        }
    }
}

/// Implement the Sv2TemplateDistributionClientHandler trait for MyTemplateDistributionClient
impl Sv2TemplateDistributionClientHandler for MyTemplateDistributionHandler {
    async fn start(&mut self) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        Ok(Sv2ClientOutcome::TriggerNewEvent(Box::new(
            Sv2ClientEvent::TemplateDistributionTrigger(
                TemplateDistributionClientTrigger::SetCoinbaseOutputConstraints(
                    self.coinbase_output_max_additional_size,
                    self.coinbase_output_max_additional_sigops,
                ),
            ),
        )))
    }

    async fn handle_new_template(
        &self,
        template: NewTemplate<'static>,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        info!("received new template: {:?}", template);
        Ok(Sv2ClientOutcome::Ok)
    }

    async fn handle_set_new_prev_hash(
        &self,
        prev_hash: SetNewPrevHash<'static>,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        info!("received new prev_hash: {:?}", prev_hash);
        Ok(Sv2ClientOutcome::Ok)
    }

    async fn handle_request_transaction_data_success(
        &self,
        transaction_data: RequestTransactionDataSuccess<'static>,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        info!(
            "received request transaction data success: {:?}",
            transaction_data
        );
        Ok(Sv2ClientOutcome::Ok)
    }

    async fn handle_request_transaction_data_error(
        &self,
        error: RequestTransactionDataError<'static>,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        info!("received request transaction data error: {:?}", error);
        Ok(Sv2ClientOutcome::Ok)
    }
}
