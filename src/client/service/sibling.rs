//! A [`Sv2SiblingServerServiceIo`] is used to send and receive requests to a sibling [`crate::client::service::Sv2ClientService`] that pairs with this server.    
//!
use crate::client::service::event::Sv2ClientEvent;
use crate::server::service::event::Sv2ServerEvent;

use async_channel::Receiver;
use async_channel::Sender;
use async_channel::TrySendError;

/// A [`Sv2SiblingServerServiceIo`] is used to send and receive requests to a sibling [`crate::client::service::Sv2ClientService`] that pairs with this server.    
#[derive(Debug, Clone)]
pub struct Sv2SiblingServerServiceIo {
    rx: Receiver<Box<Sv2ClientEvent<'static>>>,
    tx: Sender<Box<Sv2ServerEvent<'static>>>,
}

impl Sv2SiblingServerServiceIo {
    /// Create a new [`Sv2SiblingServerServiceIo`].
    ///
    /// The rx and tx are expected to be pre-built.
    pub fn set(
        rx: Receiver<Box<Sv2ClientEvent<'static>>>,
        tx: Sender<Box<Sv2ServerEvent<'static>>>,
    ) -> Self {
        Self { rx, tx }
    }

    /// Send a request to the sibling server service.
    pub fn send(
        &self,
        request: Sv2ServerEvent<'static>,
    ) -> Result<(), TrySendError<Box<Sv2ServerEvent<'static>>>> {
        self.tx.try_send(Box::new(request))
    }

    /// Receive a request from the sibling server service.
    pub async fn recv(&self) -> Result<Box<Sv2ClientEvent<'static>>, async_channel::RecvError> {
        self.rx.recv().await
    }

    /// Shutdown the sibling server service io.
    pub fn shutdown(&self) {
        self.tx.close();
        self.rx.close();
    }
}
