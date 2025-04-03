use roles_logic_sv2::mining_sv2::{
    CloseChannel, NewExtendedMiningJob, NewMiningJob, OpenExtendedMiningChannelSuccess,
    OpenMiningChannelError, OpenStandardMiningChannelSuccess, SetCustomMiningJobError,
    SetCustomMiningJobSuccess, SetExtranoncePrefix, SetGroupChannel, SetNewPrevHash,
    SetTarget, SubmitSharesError, SubmitSharesSuccess, UpdateChannelError,
};

/// Trait that must be implemented by clients that support the Mining subprotocol.
///
/// This trait defines the interface for handling all mining protocol messages in a Stratum V2 client.
/// Each handler is responsible for processing a specific message type and performing the necessary
/// actions based on the protocol specification.
///
/// The trait assumes that implementations will maintain state for each server connection, allowing
/// for proper message handling and state management across multiple connections.
pub trait Sv2MiningClientHandler: Clone + Send + Sync + 'static {
    /// Returns the subprotocol flags to be used in the SetupConnection message.
    ///
    /// These flags indicate the client's capabilities and requirements for the mining protocol.
    fn setup_connection_flags(&self) -> u32;

    /// Handles a NewMiningJob message containing a standard mining job.
    ///
    /// This message provides a new mining job with a fixed Merkle Root, where only the version,
    /// nonce, and nTime fields of the block header can be modified.
    fn handle_new_mining_job(
        &self,
        m: NewMiningJob<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles a NewExtendedMiningJob message containing an extended mining job.
    ///
    /// This message provides a mining job with a rolling Merkle Root, allowing for advanced use cases
    /// such as translation between protocols, difficulty aggregation, and search space splitting.
    fn handle_new_extended_mining_job(
        &self,
        m: NewExtendedMiningJob<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles a SetTarget message that adjusts the difficulty target for share submission.
    ///
    /// The target determines the maximum hash value that will be accepted by the server for shares.
    fn handle_set_target(
        &self,
        m: SetTarget<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles a SetNewPrevHash message indicating a new block has been detected.
    ///
    /// This message provides the previous block's hash and other necessary information for mining
    /// the next block.
    fn handle_set_new_prev_hash(
        &self,
        m: SetNewPrevHash<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles a SetCustomMiningJobSuccess message confirming a custom mining job was accepted.
    ///
    /// This message indicates that the server has accepted and will reward work on a custom mining job.
    fn handle_set_custom_mining_job_success(
        &self,
        m: SetCustomMiningJobSuccess,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles a SetCustomMiningJobError message indicating a custom mining job was rejected.
    ///
    /// This message provides information about why a custom mining job was not accepted by the server.
    fn handle_set_custom_mining_job_error(
        &self,
        m: SetCustomMiningJobError<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles an OpenMiningChannelError message indicating a channel opening failure.
    ///
    /// This message provides information about why a mining channel could not be opened.
    fn handle_open_mining_channel_error(
        &self,
        m: OpenMiningChannelError<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles an OpenStandardMiningChannelSuccess message confirming a standard channel was opened.
    ///
    /// This message provides the channel ID, target, and extranonce prefix for the new standard channel.
    fn handle_open_standard_mining_channel_success(
        &self,
        m: OpenStandardMiningChannelSuccess<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles an OpenExtendedMiningChannelSuccess message confirming an extended channel was opened.
    ///
    /// This message provides the channel ID, target, and extranonce information for the new extended channel.
    fn handle_open_extended_mining_channel_success(
        &self,
        m: OpenExtendedMiningChannelSuccess<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles a SubmitSharesSuccess message confirming shares were accepted.
    ///
    /// This message provides information about accepted shares and their sequence numbers.
    fn handle_submit_shares_success(
        &self,
        m: SubmitSharesSuccess,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles a SubmitSharesError message indicating shares were rejected.
    ///
    /// This message provides information about why submitted shares were not accepted.
    fn handle_submit_shares_error(
        &self,
        m: SubmitSharesError<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles an UpdateChannelError message indicating a channel update failure.
    ///
    /// This message provides information about why a channel update could not be processed.
    fn handle_update_channel_error(
        &self,
        m: UpdateChannelError<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles a SetGroupChannel message that associates standard channels with a group channel.
    ///
    /// This message is used for efficient job distribution to multiple standard channels at once.
    fn handle_set_group_channel(
        &self,
        m: SetGroupChannel<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles a SetExtranoncePrefix message that changes the extranonce prefix for a channel.
    ///
    /// This message updates the extranonce prefix used for all jobs sent after this message.
    fn handle_set_extranonce_prefix(
        &self,
        m: SetExtranoncePrefix<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handles a CloseChannel message indicating a channel should be closed.
    ///
    /// This message is sent when a client ends its operation or when a proxy needs to close channels.
    fn handle_close_channel(
        &self,
        m: CloseChannel<'static>,
    ) -> impl std::future::Future<Output = ()> + Send;
}

/// A null implementation of [`Sv2MiningClientHandler`] that does nothing.
///
/// This implementation is used when creating a [`crate::client::service::Sv2ClientService`] that
/// does not support the mining subprotocol. All methods will panic with an "unimplemented" message.
#[derive(Debug, Clone)]
pub struct NullSv2MiningClientHandler;

impl Sv2MiningClientHandler for NullSv2MiningClientHandler {
    fn setup_connection_flags(&self) -> u32 {
        unimplemented!("NullSv2MiningClientHandler does not implement setup_connection_flags")
    }

    async fn handle_new_mining_job(&self, _m: NewMiningJob<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_new_mining_job")
    }

    async fn handle_new_extended_mining_job(&self, _m: NewExtendedMiningJob<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_new_extended_mining_job")
    }

    async fn handle_set_target(&self, _m: SetTarget<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_set_target")
    }

    async fn handle_set_new_prev_hash(&self, _m: SetNewPrevHash<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_set_new_prev_hash")
    }

    async fn handle_set_custom_mining_job_success(&self, _m: SetCustomMiningJobSuccess) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_set_custom_mining_job_success")
    }

    async fn handle_set_custom_mining_job_error(&self, _m: SetCustomMiningJobError<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_set_custom_mining_job_error")
    }

    async fn handle_open_mining_channel_error(&self, _m: OpenMiningChannelError<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_open_mining_channel_error")
    }

    async fn handle_open_standard_mining_channel_success(&self, _m: OpenStandardMiningChannelSuccess<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_open_standard_mining_channel_success")
    }

    async fn handle_open_extended_mining_channel_success(&self, _m: OpenExtendedMiningChannelSuccess<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_open_extended_mining_channel_success")
    }

    async fn handle_submit_shares_success(&self, _m: SubmitSharesSuccess) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_submit_shares_success")
    }

    async fn handle_submit_shares_error(&self, _m: SubmitSharesError<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_submit_shares_error")
    }

    async fn handle_update_channel_error(&self, _m: UpdateChannelError<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_update_channel_error")
    }

    async fn handle_set_group_channel(&self, _m: SetGroupChannel<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_set_group_channel")
    }

    async fn handle_set_extranonce_prefix(&self, _m: SetExtranoncePrefix<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_set_extranonce_prefix")
    }

    async fn handle_close_channel(&self, _m: CloseChannel<'static>) {
        unimplemented!("NullSv2MiningClientHandler does not implement handle_close_channel")
    }
} 