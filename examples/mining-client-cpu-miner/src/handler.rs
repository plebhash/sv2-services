use roles_logic_sv2::mining_sv2::{
    CloseChannel, NewExtendedMiningJob, NewMiningJob, OpenExtendedMiningChannelSuccess,
    OpenMiningChannelError, OpenStandardMiningChannelSuccess, SetCustomMiningJobError,
    SetCustomMiningJobSuccess, SetExtranoncePrefix, SetGroupChannel, SetNewPrevHash, SetTarget,
    SubmitSharesError, SubmitSharesSuccess, UpdateChannelError,
};
use std::task::{Context, Poll};
use tower_stratum::client::service::request::RequestToSv2ClientError;
use tower_stratum::client::service::response::ResponseFromSv2Client;
use tower_stratum::client::service::subprotocols::mining::handler::Sv2MiningClientHandler;

use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct MyMiningClientHandler {
    // You could add fields here to store state or callbacks
}

impl Sv2MiningClientHandler for MyMiningClientHandler {
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), RequestToSv2ClientError>> {
        Poll::Ready(Ok(()))
    }

    async fn handle_open_standard_mining_channel_success(
        &self,
        open_standard_mining_channel_success: OpenStandardMiningChannelSuccess<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!(
            "received OpenStandardMiningChannel.Success: {:?}",
            open_standard_mining_channel_success
        );
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_open_extended_mining_channel_success(
        &self,
        open_extended_mining_channel_success: OpenExtendedMiningChannelSuccess<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!(
            "received OpenExtendedMiningChannel.Success: {:?}",
            open_extended_mining_channel_success
        );
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_open_mining_channel_error(
        &self,
        open_standard_mining_channel_error: OpenMiningChannelError<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!(
            "received OpenMiningChannel.Error: {:?}",
            open_standard_mining_channel_error
        );
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_update_channel_error(
        &self,
        update_channel_error: UpdateChannelError<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received UpdateChannel.Error: {:?}", update_channel_error);
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_close_channel(
        &self,
        close_channel: CloseChannel<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received CloseChannel: {:?}", close_channel);
        todo!()
    }

    async fn handle_set_extranonce_prefix(
        &self,
        set_extranonce_prefix: SetExtranoncePrefix<'static>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received SetExtranoncePrefix: {:?}", set_extranonce_prefix);
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_submit_shares_success(
        &self,
        submit_shares_success: SubmitSharesSuccess,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received SubmitShares.Success: {:?}", submit_shares_success);
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_submit_shares_error(
        &self,
        submit_shares_error: SubmitSharesError<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received SubmitShares.Error: {:?}", submit_shares_error);
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_new_mining_job(
        &self,
        new_mining_job: NewMiningJob<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received NewMiningJob: {:?}", new_mining_job);
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_new_extended_mining_job(
        &self,
        _new_extended_mining_job: NewExtendedMiningJob<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!(
            "received NewExtendedMiningJob: {:?}",
            _new_extended_mining_job
        );
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_set_new_prev_hash(
        &self,
        _set_new_prev_hash: SetNewPrevHash<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received SetNewPrevHash: {:?}", _set_new_prev_hash);
        // todo!()
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_set_custom_mining_job_success(
        &self,
        _set_custom_mining_job_success: SetCustomMiningJobSuccess,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("CPU Miner never sends SetCustomMiningJob");
    }

    async fn handle_set_custom_mining_job_error(
        &self,
        _set_custom_mining_job_error: SetCustomMiningJobError<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        unimplemented!("CPU Miner never sends SetCustomMiningJob");
    }

    async fn handle_set_target(
        &self,
        _set_target: SetTarget<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received SetTarget: {:?}", _set_target);
        Ok(ResponseFromSv2Client::ToDo)
    }

    async fn handle_set_group_channel(
        &self,
        _set_group_channel: SetGroupChannel<'_>,
    ) -> Result<ResponseFromSv2Client<'static>, RequestToSv2ClientError> {
        info!("received SetGroupChannel: {:?}", _set_group_channel);
        Ok(ResponseFromSv2Client::ToDo)
    }
}
