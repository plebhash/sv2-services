use crate::server::service::connection::Sv2ConnectionClient;
use crate::Sv2MessageIo;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use stratum_common::roles_logic_sv2::parsers_sv2::AnyMessage;
use tokio::sync::RwLock;

/// Representation of a Client of a Sv2 Server, to be:
/// - instantiated when a new TCP connection is established
/// - used for internal control inside [`crate::server::service::Sv2ServerService`]
#[derive(Debug)]
pub struct Sv2ServerServiceClient {
    /// The IO channels for communicating with the client
    pub io: Sv2MessageIo,
    /// The connection details, populated after a successful SetupConnection
    pub connection: RwLock<Option<Sv2ConnectionClient>>,
    /// The time of the last message received from this client (as seconds since UNIX_EPOCH)
    pub last_message_time: AtomicU64,
}

impl Sv2ServerServiceClient {
    /// Creates a new Sv2ServerServiceClient with just the IO channels
    pub fn new(io: Sv2MessageIo) -> Self {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should never go backwards")
            .as_secs();

        Self {
            io,
            connection: RwLock::new(None),
            last_message_time: AtomicU64::new(current_time),
        }
    }

    /// Updates the last_message_time to the current time
    pub fn update_last_message_time(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should never go backwards")
            .as_secs();
        self.last_message_time
            .store(current_time, Ordering::Relaxed);
    }

    /// Returns whether this client has been inactive for longer than the given duration
    pub fn is_inactive(&self, inactivity_limit_secs: u64) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should never go backwards")
            .as_secs();
        let last_message_time = self.last_message_time.load(Ordering::Relaxed);
        current_time.saturating_sub(last_message_time) > inactivity_limit_secs
    }
}

/// An ordered sequence of Sv2 messages, to be delivered to a specific client.
#[derive(Debug, Clone)]
pub struct Sv2MessagesToClient<'a> {
    pub client_id: u32,
    pub messages: Vec<AnyMessage<'a>>,
}
