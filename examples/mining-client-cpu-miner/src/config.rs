use key_utils::Secp256k1PublicKey;
use serde::Deserialize;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Deserialize)]
pub struct MyMiningClientConfig {
    pub server_addr: SocketAddr,
    pub auth_pk: Option<Secp256k1PublicKey>,
    pub n_extended_channels: u8,
    pub n_standard_channels: u8,
    pub user_identity: String,
    pub device_id: String,
}

impl MyMiningClientConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}
