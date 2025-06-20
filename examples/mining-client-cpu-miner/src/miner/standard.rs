use tower_stratum::client::service::request::RequestToSv2Client;
use tower_stratum::roles_logic_sv2::channels::client::error::StandardChannelError;
use tower_stratum::roles_logic_sv2::channels::client::standard::StandardChannel;
use tower_stratum::roles_logic_sv2::mining_sv2::{
    NewMiningJob, SetNewPrevHash, SubmitSharesStandard, Target,
};
use tower_stratum::roles_logic_sv2::{parsers::Mining, utils::u256_to_block_hash};

use bitcoin::{
    BlockHash, CompactTarget,
    blockdata::block::{Header, Version},
    hashes::sha256d::Hash,
};

use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tracing::{debug, error, info};

pub struct StandardMiner {
    standard_channel: StandardChannel<'static>,
    request_injector: async_channel::Sender<RequestToSv2Client<'static>>,
    shutdown_tx: broadcast::Sender<()>,
    is_mining: bool,
    last_share_sequence_number: Arc<Mutex<u32>>,
}

impl StandardMiner {
    pub fn new(
        standard_channel: StandardChannel<'static>,
        request_injector: async_channel::Sender<RequestToSv2Client<'static>>,
    ) -> Self {
        Self {
            standard_channel,
            request_injector,
            shutdown_tx: broadcast::channel(1).0,
            is_mining: false,
            last_share_sequence_number: Arc::new(Mutex::new(0)),
        }
    }

    pub fn shutdown(&mut self) {
        // Send shutdown signal to all tasks
        if let Err(e) = self.shutdown_tx.send(()) {
            error!("Failed to send shutdown signal: {}", e);
        }
    }

    pub fn set_extranonce_prefix(
        &mut self,
        extranonce_prefix: Vec<u8>,
    ) -> Result<(), StandardChannelError> {
        self.standard_channel
            .set_extranonce_prefix(extranonce_prefix)?;
        Ok(())
    }

    pub fn on_new_mining_job(&mut self, new_mining_job: NewMiningJob<'static>) {
        self.standard_channel
            .on_new_mining_job(new_mining_job.clone());

        // this is a non-future job
        // we should start mining immediately
        if let Some(_min_ntime) = new_mining_job.min_ntime.into_inner() {
            if self.is_mining {
                // kill task of past job
                if let Err(e) = self.shutdown_tx.send(()) {
                    error!("Failed to send shutdown signal: {}", e);
                }
            }
            // spawn task for new job
            let active_job = self
                .standard_channel
                .get_active_job()
                .expect("channel must have active job")
                .clone();
            let channel_target = self.standard_channel.get_target().clone();
            let chain_tip = self
                .standard_channel
                .get_chain_tip()
                .expect("channel must have chain tip");
            let prev_hash = u256_to_block_hash(chain_tip.prev_hash());
            let nbits = chain_tip.nbits();

            let request_injector = self.request_injector.clone();
            let shutdown_rx = self.shutdown_tx.subscribe();
            let last_share_sequence_number = self.last_share_sequence_number.clone();

            tokio::spawn(async move {
                mine_job(
                    active_job,
                    prev_hash,
                    nbits,
                    channel_target,
                    request_injector,
                    last_share_sequence_number,
                    shutdown_rx,
                )
                .await;
            });

            self.is_mining = true;
        }
    }

    pub fn on_set_new_prev_hash(
        &mut self,
        set_new_prev_hash: SetNewPrevHash<'static>,
    ) -> Result<(), StandardChannelError> {
        self.standard_channel
            .on_set_new_prev_hash(set_new_prev_hash.clone())?;

        if self.is_mining {
            // send shutdown signal to kill task of past job
            if let Err(e) = self.shutdown_tx.send(()) {
                error!("Failed to send shutdown signal: {}", e);
            }
        }

        let active_job = self
            .standard_channel
            .get_active_job()
            .expect("channel must have active job")
            .clone();
        let channel_target = self.standard_channel.get_target().clone();
        let prev_hash = u256_to_block_hash(set_new_prev_hash.prev_hash);
        let nbits = set_new_prev_hash.nbits;

        // Extract needed values from self before spawning
        let request_injector = self.request_injector.clone();
        let shutdown_rx = self.shutdown_tx.subscribe();
        let last_share_sequence_number = self.last_share_sequence_number.clone();

        tokio::spawn(async move {
            mine_job(
                active_job,
                prev_hash,
                nbits,
                channel_target,
                request_injector,
                last_share_sequence_number,
                shutdown_rx,
            )
            .await;
        });

        self.is_mining = true;

        Ok(())
    }

    pub fn set_target(&mut self, target: Target) {
        self.standard_channel.set_target(target);
    }
}

async fn mine_job(
    job: NewMiningJob<'static>,
    prevhash: BlockHash,
    nbits: u32,
    channel_target: Target,
    request_injector: async_channel::Sender<RequestToSv2Client<'static>>,
    last_share_sequence_number: Arc<Mutex<u32>>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut nonce = 0;
    let mut ntime = job
        .min_ntime
        .into_inner()
        .expect("only active jobs allowed");

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                debug!("miner task received shutdown signal... channel id: {} job id: {}", job.channel_id, job.job_id);
                break;
            }
            _ = tokio::task::yield_now() => {
                let header = Header {
                    version: Version::from_consensus(job.version as i32),
                    prev_blockhash: prevhash,
                    merkle_root: (*Hash::from_bytes_ref(
                        job.merkle_root
                            .inner_as_ref()
                            .try_into()
                            .expect("merkle_root should be 32 bytes"),
                    ))
                    .into(),
                    time: ntime,
                    bits: CompactTarget::from_consensus(nbits),
                    nonce,
                };

                // convert the header hash to a target type for easy comparison
                let hash = header.block_hash();
                let raw_hash: [u8; 32] = *hash.to_raw_hash().as_ref();
                let hash_as_target: Target = raw_hash.into();

                // is share valid?
                if hash_as_target <= channel_target {
                    let mut last_share_sequence_number = last_share_sequence_number.lock().await;

                    let share = SubmitSharesStandard {
                        channel_id: job.channel_id,
                        sequence_number: *last_share_sequence_number,
                        job_id: job.job_id,
                        nonce,
                        ntime,
                        version: job.version,
                    };

                    match request_injector.send(RequestToSv2Client::SendMessageToMiningServer(Box::new(Mining::SubmitSharesStandard(share.clone())))).await {
                        Ok(_) => {
                            info!("Submitting share: {:?}", share);
                            *last_share_sequence_number += 1;
                        }
                        Err(e) => {
                            error!("Failed to send share: {}", e);
                        }
                    }
                }

                nonce = match nonce.checked_add(1) {
                    Some(nonce) => nonce,
                    None => {
                        ntime = match ntime.checked_add(1) {
                            Some(ntime) => ntime,
                            None => {
                                error!("Both nonce and ntime overflowed");
                                break;
                            }
                        };
                        0
                    }
                };
            }
        }
    }
}
