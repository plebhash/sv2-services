use anyhow::Result;
use roles_logic_sv2::mining_sv2::{
    CloseChannel, OpenExtendedMiningChannel, OpenStandardMiningChannel, SetCustomMiningJob,
    SubmitSharesExtended, SubmitSharesStandard, UpdateChannel,
};
use roles_logic_sv2::template_distribution_sv2::{NewTemplate, SetNewPrevHash};
use tower_stratum::server::service::request::RequestToSv2ServerError;
use tower_stratum::server::service::response::ResponseFromSv2Server;
use tower_stratum::server::service::subprotocols::mining::handler::Sv2MiningServerHandler;

use std::task::{Context, Poll};
use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct MyMiningServerHandler {
    // You can add your custom fields here to store state or callbacks if needed.
}

impl Sv2MiningServerHandler for MyMiningServerHandler {
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), RequestToSv2ServerError>> {
        Poll::Ready(Ok(()))
    }

    async fn on_new_template(
        &self,
        m: NewTemplate<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        info!(
            template_id = m.template_id,
            "MiningServer: Received new template"
        );

        // Store the latest template ID for assertions in tests

        Ok(ResponseFromSv2Server::Ok)
    }

    async fn on_set_new_prev_hash(
        &self,
        m: SetNewPrevHash<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        info!(prev_hash = ?m.prev_hash, "MiningServer: Received new previous hash");
        Ok(ResponseFromSv2Server::Ok)
    }

    fn setup_connection_success_flags(&self) -> u32 {
        0
    }

    async fn add_client(&mut self, client_id: u32, flags: u32) {
        info!("adding client with id: {}, flags: {}", client_id, flags);
    }

    async fn remove_client(&mut self, client_id: u32) {
        info!("removing client with id: {}", client_id);
    }

    async fn remove_all_clients(&mut self) {
        info!("removing all clients");
    }

    async fn handle_open_standard_mining_channel(
        &self,
        _client_id: u32,
        _m: OpenStandardMiningChannel<'static>,
    ) -> Result<ResponseFromSv2Server<'static>, RequestToSv2ServerError> {
        unimplemented!(
            "MyMiningServerHandler does not implement handle_open_standard_mining_channel"
        )
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
}
