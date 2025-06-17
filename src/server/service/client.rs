use crate::server::service::connection::Sv2ConnectionClient;
use crate::Sv2MessageIo;
use roles_logic_sv2::parsers::AnyMessage;
use std::time::Instant;

/// Representation of a Client of a Sv2 Server, to be:
/// - instantiated when a new TCP connection is established
/// - used for internal control inside [`crate::server::service::Sv2ServerService`]
#[derive(Debug, Clone)]
pub struct Sv2ServerServiceClient {
    /// The IO channels for communicating with the client
    pub io: Sv2MessageIo,
    /// The connection details, populated after a successful SetupConnection
    pub connection: Option<Sv2ConnectionClient>,
    /// The time of the last message received from this client
    pub last_message_instant: Instant,
}

impl Sv2ServerServiceClient {
    /// Creates a new Sv2ServerServiceClient with just the IO channels
    pub fn new(io: Sv2MessageIo) -> Self {
        Self {
            io,
            connection: None,
            last_message_instant: Instant::now(),
        }
    }

    /// Updates the last_message_instant to the current time
    pub fn update_last_message_time(&mut self) {
        self.last_message_instant = Instant::now();
    }

    /// Returns whether this client has been inactive for longer than the given duration
    pub fn is_inactive(&self, inactivity_limit_secs: u64) -> bool {
        Instant::now()
            .duration_since(self.last_message_instant)
            .as_secs()
            > inactivity_limit_secs
    }
}

/// An ordered sequence of Sv2 messages, to be delivered to a specific client.
#[derive(Debug, Clone)]
pub struct Sv2MessagesToClient<'a> {
    pub client_id: u32,
    pub messages: Vec<AnyMessage<'a>>,
}
