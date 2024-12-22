use crate::client::service::request::RequestToSv2ClientError;
use crate::client::service::response::ResponseFromSv2Client;

use roles_logic_sv2::parsers::{AnyMessage, TemplateDistribution};
use roles_logic_sv2::template_distribution_sv2::{
    CoinbaseOutputConstraints, NewTemplate, RequestTransactionData, RequestTransactionDataError,
    RequestTransactionDataSuccess, SetNewPrevHash,
};

/// Trait that must be implemented in case [`crate::client::service::Sv2ClientService`] supports the Template Distribution protocol
pub trait Sv2TemplateDistributionClientHandler {
    fn handle_new_template(
        &self,
        template: NewTemplate<'static>,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_set_new_prev_hash(
        &self,
        prev_hash: SetNewPrevHash<'static>,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_request_transaction_data_success(
        &self,
        transaction_data: RequestTransactionDataSuccess<'static>,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_request_transaction_data_error(
        &self,
        error: RequestTransactionDataError<'static>,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn transaction_data_needed(
        &self,
        template_id: u64,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send {
        let message = AnyMessage::TemplateDistribution(
            TemplateDistribution::RequestTransactionData(RequestTransactionData { template_id }),
        );

        async move { Ok(ResponseFromSv2Client::SendToServer(message)) }
    }

    fn set_coinbase_output_constraints(
        &self,
        coinbase_output_max_additional_size: u32,
        coinbase_output_max_additional_sigops: u16,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send {
        let message = AnyMessage::TemplateDistribution(
            TemplateDistribution::CoinbaseOutputConstraints(CoinbaseOutputConstraints {
                coinbase_output_max_additional_size,
                coinbase_output_max_additional_sigops,
            }),
        );

        async move { Ok(ResponseFromSv2Client::SendToServer(message)) }
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
    async fn handle_new_template(
        &self,
        _template: NewTemplate<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!(
            "NullSv2TemplateDistributionClientHandler does not implement handle_new_template"
        );
    }

    async fn handle_set_new_prev_hash(
        &self,
        _prev_hash: SetNewPrevHash<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!(
            "NullSv2TemplateDistributionClientHandler does not implement handle_set_new_prev_hash"
        );
    }

    async fn handle_request_transaction_data_success(
        &self,
        _transaction_data: RequestTransactionDataSuccess<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2TemplateDistributionClientHandler does not implement handle_request_transaction_data_success");
    }

    async fn handle_request_transaction_data_error(
        &self,
        _error: RequestTransactionDataError<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2TemplateDistributionClientHandler does not implement handle_request_transaction_data_error");
    }

    async fn transaction_data_needed(
        &self,
        _template_id: u64,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!(
            "NullSv2TemplateDistributionClientHandler does not implement request_transaction_data"
        );
    }

    async fn set_coinbase_output_constraints(
        &self,
        _coinbase_output_max_additional_size: u32,
        _coinbase_output_max_additional_sigops: u16,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2TemplateDistributionClientHandler does not implement send_coinbase_output_constraints");
    }
}
