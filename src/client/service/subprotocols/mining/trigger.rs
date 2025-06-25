#[derive(Debug, Clone)]
pub enum MiningClientTrigger {
    // Start the mining handler
    Start,
    // request_id, user_identity, nominal_hashrate, max_target
    OpenStandardMiningChannel(u32, String, f32, Vec<u8>),
    // request_id, user_identity, nominal_hashrate, max_target, min_rollable_extranonce_size
    OpenExtendedMiningChannel(u32, String, f32, Vec<u8>, u16),
}
