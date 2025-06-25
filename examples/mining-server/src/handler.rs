use anyhow::Result;
use dashmap::DashMap;
use stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::B032;
use stratum_common::roles_logic_sv2::codec_sv2::binary_sv2::U256;
use stratum_common::roles_logic_sv2::mining_sv2::{
    CloseChannel, OpenExtendedMiningChannel, OpenStandardMiningChannel,
    OpenStandardMiningChannelSuccess, SetCustomMiningJob, SubmitSharesExtended,
    SubmitSharesStandard, UpdateChannel,
};
use stratum_common::roles_logic_sv2::parsers::{AnyMessage, Mining};
use stratum_common::roles_logic_sv2::template_distribution_sv2::{NewTemplate, SetNewPrevHash};
use std::sync::Arc;
use tower_stratum::server::service::client::Sv2MessagesToClient;
use tower_stratum::server::service::request::{RequestToSv2Server, RequestToSv2ServerError};
use tower_stratum::server::service::response::ResponseFromSv2Server;
use tower_stratum::server::service::subprotocols::mining::handler::Sv2MiningServerHandler;

use crate::client::MyMiningServerClient;

use std::task::{Context, Poll};
use tracing::info;
#[derive(Debug, Clone, Default)]
pub struct MyMiningServerHandler {
    clients: Arc<DashMap<u32, MyMiningServerClient>>,
}

impl Sv2MiningServerHandler for MyMiningServerHandler {
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), RequestToSv2ServerError>> {
        Poll::Ready(Ok(()))
    }

    async fn start(&mut self) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        Ok(ResponseFromSv2Server::Ok)
    }

    // no spawned tasks, therefore empty shutdown method
    async fn shutdown(&mut self) {}

    fn setup_connection_success_flags(&self) -> u32 {
        0
    }

    async fn add_client(&mut self, client_id: u32, flags: u32) {
        info!("adding client with id: {}, flags: {}", client_id, flags);
        self.clients
            .insert(client_id, MyMiningServerClient { _flags: flags });
    }

    async fn remove_client(&mut self, client_id: u32) {
        info!("removing client with id: {}", client_id);
        self.clients.remove(&client_id);
    }

    async fn handle_open_standard_mining_channel(
        &self,
        client_id: u32,
        m: OpenStandardMiningChannel<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        info!("received OpenStandardMiningChannel");

        let request_id = m.request_id;

        // Convert hex string to U256
        let target_hex = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let target_bytes = hex::decode(target_hex).expect("Invalid hex");
        let target_bytes_array: [u8; 32] = target_bytes.try_into().expect("Expected 32 bytes");
        let target = U256::from(target_bytes_array);

        // assigns unique extranonce prefix for the client - using client_id in first 4 bytes
        let mut extranonce_prefix = vec![0u8; 32];
        extranonce_prefix[0..4].copy_from_slice(&client_id.to_be_bytes());
        info!(
            "extranonce_prefix for client {}: 0x{}",
            client_id,
            hex::encode(&extranonce_prefix)
        );
        let extranonce_prefix: B032 = extranonce_prefix.try_into().expect("Expected 32 bytes");

        // todo: update some actual state on the server representing this new standard mining channel

        let message = Sv2MessagesToClient {
            client_id,
            messages: vec![AnyMessage::Mining(
                Mining::OpenStandardMiningChannelSuccess(OpenStandardMiningChannelSuccess {
                    request_id,
                    channel_id: 0,
                    target,
                    extranonce_prefix,
                    group_channel_id: 0,
                }),
            )],
        };
        info!(
            "sending OpenStandardMiningChannelSuccess to client with id: {}",
            client_id
        );
        Ok(ResponseFromSv2Server::TriggerNewRequest(Box::new(
            RequestToSv2Server::SendMessagesToClient(Box::new(message)),
        )))
    }

    async fn handle_open_extended_mining_channel(
        &self,
        _client_id: u32,
        _m: OpenExtendedMiningChannel<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        unimplemented!(
            "MyMiningServerHandler does not implement handle_open_extended_mining_channel"
        )
    }

    async fn handle_update_channel(
        &self,
        _client_id: u32,
        _m: UpdateChannel<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        unimplemented!("MyMiningServerHandler does not implement handle_update_channel")
    }

    async fn handle_close_channel(
        &self,
        _client_id: u32,
        _m: CloseChannel<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        unimplemented!("MyMiningServerHandler does not implement handle_close_channel")
    }

    async fn handle_submit_shares_standard(
        &self,
        _client_id: u32,
        _m: SubmitSharesStandard,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        unimplemented!("MyMiningServerHandler does not implement handle_submit_shares_standard")
    }

    async fn handle_submit_shares_extended(
        &self,
        _client_id: u32,
        _m: SubmitSharesExtended<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        unimplemented!("MyMiningServerHandler does not implement handle_submit_shares_extended")
    }

    async fn handle_set_custom_mining_job(
        &self,
        _client_id: u32,
        _m: SetCustomMiningJob<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        unimplemented!("MyMiningServerHandler does not implement handle_set_custom_mining_job")
    }

    async fn on_new_template(
        &self,
        _m: NewTemplate<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        unimplemented!("MyMiningServerHandler does not implement on_new_template")
    }

    async fn on_set_new_prev_hash(
        &self,
        _m: SetNewPrevHash<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        unimplemented!("MyMiningServerHandler does not implement on_set_new_prev_hash")
    }
}
