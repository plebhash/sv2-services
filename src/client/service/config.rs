use key_utils::Secp256k1PublicKey;
use roles_logic_sv2::common_messages_sv2::Protocol;
use std::net::SocketAddr;

/// Configuration for [`crate::client::service::Sv2ClientService`]
#[derive(Debug, Clone)]
pub struct Sv2ClientServiceConfig {
    /// The minimum protocol version this client is willing to speak.
    pub min_supported_version: u16,
    /// The maximum protocol version this client is willing to speak.
    pub max_supported_version: u16,
    /// (Optional) ASCII text indicating the hostname or IP address
    pub endpoint_host: Option<String>,
    /// (Optional) Connecting port
    pub endpoint_port: Option<u16>,
    /// (Optional) ASCII text indicating the vendor name
    pub vendor: Option<String>,
    /// (Optional) ASCII text indicating the hardware version
    pub hardware_version: Option<String>,
    /// (Optional) ASCII text indicating the firmware version
    pub firmware: Option<String>,
    /// (Optional) ASCII text indicating the device ID
    pub device_id: Option<String>,
    /// Configuration specific to the Mining protocol
    pub mining_config: Option<Sv2ClientServiceMiningConfig>,
    /// Configuration specific to the Job Declaration protocol
    pub job_declaration_config: Option<Sv2ClientServiceJobDeclarationConfig>,
    /// Configuration specific to the Template Distribution protocol
    pub template_distribution_config: Option<Sv2ClientServiceTemplateDistributionConfig>,
}

impl Sv2ClientServiceConfig {
    /// Returns the list of:
    /// - supported protocols based on the presence of config fields.
    /// - flags to be sent in the SetupConnection message, for each supported protocol.
    pub fn supported_protocols(&self) -> Vec<(Protocol, u32)> {
        let mut protocols = Vec::new();

        if self.mining_config.is_some() {
            let flags = self
                .mining_config
                .as_ref()
                .expect("mining_config must be Some")
                .setup_connection_flags;
            protocols.push((Protocol::MiningProtocol, flags));
        }

        if self.job_declaration_config.is_some() {
            let flags = self
                .job_declaration_config
                .as_ref()
                .expect("job_declaration_config must be Some")
                .setup_connection_flags;
            protocols.push((Protocol::JobDeclarationProtocol, flags));
        }

        if self.template_distribution_config.is_some() {
            let flags = self
                .template_distribution_config
                .as_ref()
                .expect("template_distribution_config must be Some")
                .setup_connection_flags;
            protocols.push((Protocol::TemplateDistributionProtocol, flags));
        }

        protocols
    }
}

/// Configuration in case Sv2ClientService supports the Template Distribution protocol
#[derive(Debug, Clone)]
pub struct Sv2ClientServiceTemplateDistributionConfig {
    /// The server address to connect to
    pub server_addr: SocketAddr,
    /// Optional authentication public key for encrypted connections
    pub auth_pk: Option<Secp256k1PublicKey>,
    /// Coinbase output constraints in the format (max_additional_size, max_additional_sigops)
    pub coinbase_output_constraints: (u32, u16),
    /// Flags to be sent in the SetupConnection message
    pub setup_connection_flags: u32,
}

/// Configuration in case Sv2ClientService supports the Job Declaration protocol
#[derive(Debug, Clone)]
pub struct Sv2ClientServiceJobDeclarationConfig {
    /// The server address to connect to
    pub server_addr: SocketAddr,
    /// Optional authentication public key for encrypted connections
    pub auth_pk: Option<Secp256k1PublicKey>,
    /// Flags to be sent in the SetupConnection message
    pub setup_connection_flags: u32,
}

/// Configuration in case Sv2ClientService supports the Mining protocol
#[derive(Debug, Clone)]
pub struct Sv2ClientServiceMiningConfig {
    /// The server address to connect to
    pub server_addr: SocketAddr,
    /// Optional authentication public key for encrypted connections
    pub auth_pk: Option<Secp256k1PublicKey>,
    /// Flags to be sent in the SetupConnection message
    pub setup_connection_flags: u32,
}
