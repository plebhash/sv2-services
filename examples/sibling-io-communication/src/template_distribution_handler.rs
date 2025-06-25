use anyhow::Result;
use stratum_common::roles_logic_sv2::template_distribution_sv2::{NewTemplate, SetNewPrevHash};

use std::task::{Context, Poll};
use stratum_common::roles_logic_sv2::template_distribution_sv2::{
    RequestTransactionDataError, RequestTransactionDataSuccess,
};
use tower_stratum::client::service::request::{RequestToSv2Client, RequestToSv2ClientError};
use tower_stratum::client::service::response::ResponseFromSv2Client;
use tower_stratum::client::service::subprotocols::template_distribution::handler::Sv2TemplateDistributionClientHandler;
use tower_stratum::client::service::subprotocols::template_distribution::trigger::TemplateDistributionClientTrigger;
use tower_stratum::server::service::request::RequestToSv2Server;
use tower_stratum::server::service::subprotocols::mining::trigger::MiningServerTrigger;
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
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), RequestToSv2ClientError>> {
        // Indicates that the handler is ready to process requests.
        Poll::Ready(Ok(()))
    }

    async fn start(&mut self) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        Ok(ResponseFromSv2Client::TriggerNewRequest(Box::new(
            RequestToSv2Client::TemplateDistributionTrigger(
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
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!(
            template_id = template.template_id,
            "TDC: Forwarding new template to MiningServer"
        );

        // This is where the SiblingIO mechanism is utilized.
        // The new template is forwarded to the MiningServer using a sibling request.
        // For more details on SiblingIO, refer to the following link:
        // https://github.com/plebhash/tower-stratum/blob/de6d909bae4c85e19b75b20081b921efb6830d81/src/client/service/mod.rs#L774-L781
        // SiblingIO works by recursively triggering requests. In this case, a new request of type
        // `RequestToSv2Client::SendRequestToSiblingServerService` is created and dispatched to the MiningServer.
        // The MiningServer, being a sibling server of the TDC Handler, processes the request accordingly.

        let response = ResponseFromSv2Client::TriggerNewRequest(Box::new(
            RequestToSv2Client::SendRequestToSiblingServerService(Box::new(
                RequestToSv2Server::MiningTrigger(MiningServerTrigger::NewTemplate(template)),
            )),
        ));
        Ok(response)
    }

    async fn handle_set_new_prev_hash(
        &self,
        prev_hash: SetNewPrevHash<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!(prev_hash = ?prev_hash, "MiningServer: Received new previous hash");

        // Similar to `handle_new_template`, this forwards the new previous hash to the MiningServer.
        let response = ResponseFromSv2Client::TriggerNewRequest(Box::new(
            RequestToSv2Client::SendRequestToSiblingServerService(Box::new(
                RequestToSv2Server::MiningTrigger(MiningServerTrigger::SetNewPrevHash(prev_hash)),
            )),
        ));
        Ok(response)
    }

    async fn handle_request_transaction_data_success(
        &self,
        transaction_data: RequestTransactionDataSuccess<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!(
            "received request transaction data success: {:?}",
            transaction_data
        );
        Ok(ResponseFromSv2Client::Ok)
    }

    async fn handle_request_transaction_data_error(
        &self,
        error: RequestTransactionDataError<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received request transaction data error: {:?}", error);
        Ok(ResponseFromSv2Client::Ok)
    }
}
