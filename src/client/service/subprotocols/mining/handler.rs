use crate::client::service::request::RequestToSv2ClientError;
use crate::client::service::response::ResponseFromSv2Client;

use stratum_common::roles_logic_sv2::mining_sv2::{
    CloseChannel, NewExtendedMiningJob, NewMiningJob, OpenExtendedMiningChannelSuccess,
    OpenMiningChannelError, OpenStandardMiningChannelSuccess, SetCustomMiningJobError,
    SetCustomMiningJobSuccess, SetExtranoncePrefix, SetGroupChannel, SetNewPrevHash, SetTarget,
    SubmitSharesError, SubmitSharesSuccess, UpdateChannelError,
};

use std::task::{Context, Poll};

/// Trait that must be implemented in case [`crate::client::service::Sv2ClientService`] supports the Mining protocol
pub trait Sv2MiningClientHandler {
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), RequestToSv2ClientError>>;

    fn start(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_open_standard_mining_channel_success(
        &mut self,
        open_standard_mining_channel_success: OpenStandardMiningChannelSuccess<'static>,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_open_extended_mining_channel_success(
        &mut self,
        open_extended_mining_channel_success: OpenExtendedMiningChannelSuccess<'static>,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_open_mining_channel_error(
        &mut self,
        open_mining_channel_error: OpenMiningChannelError<'static>,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_update_channel_error(
        &mut self,
        update_channel_error: UpdateChannelError<'static>,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_close_channel(
        &mut self,
        close_channel: CloseChannel<'static>,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_set_extranonce_prefix(
        &mut self,
        set_extranonce_prefix: SetExtranoncePrefix<'static>,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_submit_shares_success(
        &mut self,
        submit_shares_success: SubmitSharesSuccess,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_submit_shares_error(
        &mut self,
        submit_shares_error: SubmitSharesError,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_new_mining_job(
        &mut self,
        new_mining_job: NewMiningJob,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_new_extended_mining_job(
        &mut self,
        new_extended_mining_job: NewExtendedMiningJob,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_set_new_prev_hash(
        &mut self,
        set_new_prev_hash: SetNewPrevHash,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_set_custom_mining_job_success(
        &mut self,
        set_custom_mining_job_success: SetCustomMiningJobSuccess,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_set_custom_mining_job_error(
        &mut self,
        set_custom_mining_job_error: SetCustomMiningJobError,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_set_target(
        &mut self,
        set_target: SetTarget,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;

    fn handle_set_group_channel(
        &mut self,
        set_group_channel: SetGroupChannel,
    ) -> impl std::future::Future<
        Output = Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError>,
    > + Send;
}

// -------------------------------------------------------------------------------------------------
// NullSv2MiningClientHandler
// -------------------------------------------------------------------------------------------------

/// A [`Sv2MiningClientHandler`] implementation that does nothing.
///
/// It should be used when creating a [`crate::client::service::Sv2ClientService`] that
/// does not support the Mining protocol.
#[derive(Debug, Clone)]
pub struct NullSv2MiningClientHandler;

impl Sv2MiningClientHandler for NullSv2MiningClientHandler {
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), RequestToSv2ClientError>> {
        unimplemented!("NullSv2TemplateDistributionClientHandler does not implement poll_ready");
    }

    async fn start(&mut self) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement start");
    }

    async fn handle_open_standard_mining_channel_success(
        &mut self,
        _open_standard_mining_channel_success: OpenStandardMiningChannelSuccess<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_open_standard_mining_channel_success");
    }

    async fn handle_open_extended_mining_channel_success(
        &mut self,
        _open_extended_mining_channel_success: OpenExtendedMiningChannelSuccess<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_open_extended_mining_channel_success");
    }

    async fn handle_open_mining_channel_error(
        &mut self,
        _open_standard_mining_channel_error: OpenMiningChannelError<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_open_standard_mining_channel_error");
    }

    async fn handle_update_channel_error(
        &mut self,
        _update_channel_error: UpdateChannelError<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_update_channel_error");
    }

    async fn handle_close_channel(
        &mut self,
        _close_channel: CloseChannel<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_close_channel");
    }

    async fn handle_set_extranonce_prefix(
        &mut self,
        _set_extranonce_prefix: SetExtranoncePrefix<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!(
            "NullSv2MiningClientHandler does not implement handle_set_extranonce_prefix"
        );
    }

    async fn handle_submit_shares_success(
        &mut self,
        _submit_shares_success: SubmitSharesSuccess,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!(
            "NullSv2MiningClientHandler does not implement handle_submit_shares_success"
        );
    }

    async fn handle_submit_shares_error(
        &mut self,
        _submit_shares_error: SubmitSharesError<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_submit_shares_error");
    }

    async fn handle_new_mining_job(
        &mut self,
        _new_mining_job: NewMiningJob<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_new_mining_job");
    }

    async fn handle_new_extended_mining_job(
        &mut self,
        _new_extended_mining_job: NewExtendedMiningJob<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!(
            "NullSv2MiningClientHandler does not implement handle_new_extended_mining_job"
        );
    }

    async fn handle_set_new_prev_hash(
        &mut self,
        _set_new_prev_hash: SetNewPrevHash<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_set_new_prev_hash");
    }

    async fn handle_set_custom_mining_job_success(
        &mut self,
        _set_custom_mining_job_success: SetCustomMiningJobSuccess,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!(
            "NullSv2MiningClientHandler does not implement handle_set_custom_mining_job_success"
        );
    }

    async fn handle_set_custom_mining_job_error(
        &mut self,
        _set_custom_mining_job_error: SetCustomMiningJobError<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!(
            "NullSv2MiningClientHandler does not implement handle_set_custom_mining_job_error"
        );
    }

    async fn handle_set_target(
        &mut self,
        _set_target: SetTarget<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_set_target");
    }

    async fn handle_set_group_channel(
        &mut self,
        _set_group_channel: SetGroupChannel<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_set_group_channel");
    }
}
