#[derive(Debug, Clone)]
pub enum RequestToSv2MiningClientService {
    OpenStandardMiningChannel(u32, String, f32, Vec<u8>),
    OpenExtendedMiningChannel(u32, String, f32, Vec<u8>, u16),
}
