use anyhow::Result;
use sv2_services::roles_logic_sv2::mining_sv2::{
    CloseChannel, OpenExtendedMiningChannel, OpenStandardMiningChannel, SetCustomMiningJob,
    SubmitSharesExtended, SubmitSharesStandard, UpdateChannel,
};
use sv2_services::roles_logic_sv2::template_distribution_sv2::{NewTemplate, SetNewPrevHash};
use sv2_services::server::service::event::Sv2ServerEventError;
use sv2_services::server::service::outcome::Sv2ServerOutcome;
use sv2_services::server::service::subprotocols::mining::handler::Sv2MiningServerHandler;

use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct MyMiningServerHandler {
    // You can add your custom fields here to store state or callbacks if needed.
}

impl Sv2MiningServerHandler for MyMiningServerHandler {
    async fn start(&mut self) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
        Ok(Sv2ServerOutcome::Ok)
    }

    async fn on_new_template(
        &self,
        m: NewTemplate<'static>,
    ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
        info!(
            template_id = m.template_id,
            "MiningServer: Received new template"
        );

        Ok(Sv2ServerOutcome::Ok)
    }

    async fn on_set_new_prev_hash(
        &self,
        m: SetNewPrevHash<'static>,
    ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
        info!(prev_hash = ?m.prev_hash, "MiningServer: Received new previous hash");
        Ok(Sv2ServerOutcome::Ok)
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

    async fn handle_open_standard_mining_channel(
        &self,
        _client_id: u32,
        _m: OpenStandardMiningChannel<'static>,
    ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
        info!("MiningServer: Received OpenStandardMiningChannel");
        Ok(Sv2ServerOutcome::Ok)
    }

    async fn handle_open_extended_mining_channel(
        &self,
        _client_id: u32,
        _m: OpenExtendedMiningChannel<'static>,
    ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
        info!("MiningServer: Received OpenExtendedMiningChannel");
        Ok(Sv2ServerOutcome::Ok)
    }

    async fn handle_update_channel(
        &self,
        _client_id: u32,
        _m: UpdateChannel<'static>,
    ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
        info!("MiningServer: Received UpdateChannel");
        Ok(Sv2ServerOutcome::Ok)
    }

    async fn handle_close_channel(
        &self,
        _client_id: u32,
        _m: CloseChannel<'static>,
    ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
        info!("MiningServer: Received CloseChannel");
        Ok(Sv2ServerOutcome::Ok)
    }

    async fn handle_submit_shares_standard(
        &self,
        _client_id: u32,
        _m: SubmitSharesStandard,
    ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
        info!("MiningServer: Received SubmitSharesStandard");
        Ok(Sv2ServerOutcome::Ok)
    }

    async fn handle_submit_shares_extended(
        &self,
        _client_id: u32,
        _m: SubmitSharesExtended<'static>,
    ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
        info!("MiningServer: Received SubmitSharesExtended");
        Ok(Sv2ServerOutcome::Ok)
    }

    async fn handle_set_custom_mining_job(
        &self,
        _client_id: u32,
        _m: SetCustomMiningJob<'static>,
    ) -> Result<Sv2ServerOutcome<'static>, Sv2ServerEventError> {
        info!("MiningServer: Received SetCustomMiningJob");
        Ok(Sv2ServerOutcome::Ok)
    }
}
