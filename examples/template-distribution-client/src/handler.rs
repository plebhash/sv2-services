use anyhow::Result;
use roles_logic_sv2::template_distribution_sv2::{
    NewTemplate, RequestTransactionDataError, RequestTransactionDataSuccess, SetNewPrevHash,
};
use tower_stratum::client::service::request::RequestToSv2ClientError;
use tower_stratum::client::service::response::ResponseFromSv2Client;
use tower_stratum::client::service::subprotocols::template_distribution::handler::Sv2TemplateDistributionClientHandler;
use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct MyTemplateDistributionHandler {
    // You could add fields here to store state or callbacks
}

/// Implement the Sv2TemplateDistributionClientHandler trait for MyTemplateDistributionClient
impl Sv2TemplateDistributionClientHandler for MyTemplateDistributionHandler {
    async fn handle_new_template(
        &self,
        template: NewTemplate<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received new template: {:?}", template);
        Ok(ResponseFromSv2Client::Ok)
    }

    async fn handle_set_new_prev_hash(
        &self,
        prev_hash: SetNewPrevHash<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received new prev_hash: {:?}", prev_hash);
        Ok(ResponseFromSv2Client::Ok)
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
