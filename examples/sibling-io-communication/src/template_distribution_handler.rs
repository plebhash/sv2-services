use anyhow::Result;
use sv2_services::roles_logic_sv2::template_distribution_sv2::{NewTemplate, SetNewPrevHash};

use sv2_services::client::service::event::Sv2ClientEvent;
use sv2_services::client::service::event::Sv2ClientEventError;
use sv2_services::client::service::outcome::Sv2ClientOutcome;
use sv2_services::client::service::subprotocols::template_distribution::handler::Sv2TemplateDistributionClientHandler;
use sv2_services::client::service::subprotocols::template_distribution::trigger::TemplateDistributionClientTrigger;
use sv2_services::roles_logic_sv2::template_distribution_sv2::{
    RequestTransactionDataError, RequestTransactionDataSuccess,
};
use sv2_services::server::service::event::Sv2ServerEvent;
use sv2_services::server::service::subprotocols::mining::trigger::MiningServerTrigger;
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

/// Implements the `Sv2TemplateDistributionClientHandler` trait for `MyTemplateDistributionHandler`.
/// This trait defines how the handler processes incoming template distribution events.
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
        info!(
            template_id = template.template_id,
            "TDC: Forwarding new template to MiningServer"
        );

        // This is where the SiblingIO mechanism is utilized.
        // The new template is forwarded to the MiningServer using a sibling event.
        // SiblingIO works by recursively triggering events. In this case, a new event of type
        // `Sv2ClientEvent::SendEventToSiblingServerService` is created and dispatched to the MiningServer.
        // The MiningServer, being a sibling server of the TDC Handler, processes the event accordingly.

        let outcome = Sv2ClientOutcome::TriggerNewEvent(Box::new(
            Sv2ClientEvent::SendEventToSiblingServerService(Box::new(
                Sv2ServerEvent::MiningTrigger(MiningServerTrigger::NewTemplate(template)),
            )),
        ));
        Ok(outcome)
    }

    async fn handle_set_new_prev_hash(
        &self,
        prev_hash: SetNewPrevHash<'static>,
    ) -> Result<Sv2ClientOutcome<'static>, Sv2ClientEventError> {
        info!(prev_hash = ?prev_hash, "MiningServer: Received new previous hash");

        // Similar to `handle_new_template`, this forwards the new previous hash to the MiningServer.
        let outcome = Sv2ClientOutcome::TriggerNewEvent(Box::new(
            Sv2ClientEvent::SendEventToSiblingServerService(Box::new(
                Sv2ServerEvent::MiningTrigger(MiningServerTrigger::SetNewPrevHash(prev_hash)),
            )),
        ));
        Ok(outcome)
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
