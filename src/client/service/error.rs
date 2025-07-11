use std::fmt;
use stratum_common::roles_logic_sv2::common_messages_sv2::Protocol;

#[derive(Debug)]
pub enum Sv2ClientServiceError {
    IsNotConnected,
    BadConfig,
    NullHandlerForSupportedProtocol { protocol: Protocol },
    NonNullHandlerForUnsupportedProtocol { protocol: Protocol },
    ServiceNotReady,
    FailedToInitiateConnection { protocol: Protocol },
    FailedToStartMiningHandler,
    // FailedToStartJobDeclarationHandler,
    FailedToStartTemplateDistributionHandler,
    NoSiblingServerServiceIo,
}

impl fmt::Display for Sv2ClientServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sv2ClientServiceError::IsNotConnected => write!(f, "Is not connected"),
            Sv2ClientServiceError::BadConfig => write!(f, "Bad config"),
            Sv2ClientServiceError::NullHandlerForSupportedProtocol { protocol } => {
                write!(f, "Null handler for supported protocol {protocol:?}")
            }
            Sv2ClientServiceError::NonNullHandlerForUnsupportedProtocol { protocol } => {
                write!(f, "Non-null handler for unsupported protocol {protocol:?}")
            }
            Sv2ClientServiceError::ServiceNotReady => write!(f, "Service not ready"),
            Sv2ClientServiceError::FailedToInitiateConnection { protocol } => {
                write!(f, "Failed to initiate connection with {protocol:?}")
            }
            Sv2ClientServiceError::FailedToStartMiningHandler => {
                write!(f, "Failed to start mining handler")
            }
            // Sv2ClientServiceError::FailedToStartJobDeclarationHandler => {
            //     write!(f, "Failed to start job declaration handler")
            // }
            Sv2ClientServiceError::FailedToStartTemplateDistributionHandler => {
                write!(f, "Failed to start template distribution handler")
            }
            Sv2ClientServiceError::NoSiblingServerServiceIo => {
                write!(f, "No sibling server service io")
            }
        }
    }
}

impl std::error::Error for Sv2ClientServiceError {}
