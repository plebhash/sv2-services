use roles_logic_sv2::common_messages_sv2::Protocol;
use std::fmt;

#[derive(Debug)]
pub enum Sv2ClientServiceError {
    BadConfig,
    NullHandlerForSupportedProtocol { protocol: Protocol },
    NonNullHandlerForUnsupportedProtocol { protocol: Protocol },
    ServiceNotReady,
    FailedToInitiateConnection { protocol: Protocol },
}

impl fmt::Display for Sv2ClientServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sv2ClientServiceError::BadConfig => write!(f, "Bad config"),
            Sv2ClientServiceError::NullHandlerForSupportedProtocol { protocol } => {
                write!(f, "Null handler for supported protocol {:?}", protocol)
            }
            Sv2ClientServiceError::NonNullHandlerForUnsupportedProtocol { protocol } => write!(
                f,
                "Non-null handler for unsupported protocol {:?}",
                protocol
            ),
            Sv2ClientServiceError::ServiceNotReady => write!(f, "Service not ready"),
            Sv2ClientServiceError::FailedToInitiateConnection { protocol } => {
                write!(f, "Failed to initiate connection with {:?}", protocol)
            }
        }
    }
}

impl std::error::Error for Sv2ClientServiceError {}
