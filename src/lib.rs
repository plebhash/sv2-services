//! # Introduction
//!
//! This crate aims to provide a Service-Oriented framework for building Bitcoin mining apps based on:
//!- [Tokio](https://tokio.rs/): an asynchronous runtime for Rust
//!- [Stratum V2 Reference Implementation](https://github.com/stratum-mining/stratum): the reference implementation of the Stratum V2 protocol
//!
//! The goal is to provide a "batteries-included" approach to implement stateful Sv2 applications.

use async_channel::{Receiver, Sender};
use stratum_common::roles_logic_sv2::{
    codec_sv2::{framing_sv2::framing::Frame, StandardEitherFrame, StandardSv2Frame},
    parsers_sv2::{
        AnyMessage, CommonMessages,
        JobDeclaration::{
            AllocateMiningJobToken, AllocateMiningJobTokenSuccess, DeclareMiningJob,
            DeclareMiningJobError, DeclareMiningJobSuccess, ProvideMissingTransactions,
            ProvideMissingTransactionsSuccess, PushSolution,
        },
        TemplateDistribution::{self, CoinbaseOutputConstraints},
    },
};

use std::future::Future;

pub use key_utils;
pub use stratum_common::roles_logic_sv2;

/// Client-side modules for establishing and managing Stratum V2 connections.
///
/// This module provides the client service implementation of the Stratum V2 protocol, including:
/// - TCP connection handlers (both encrypted and unencrypted)
/// - Service implementations for different Stratum V2 subprotocols
/// - Configuration options for client behavior
pub mod client;

/// Server-side modules for accepting and handling Stratum V2 client connections.
///
/// This module provides the server service implementation of the Stratum V2 protocol, including:
/// - TCP connection handlers (both encrypted and unencrypted)
/// - Service implementations for different Stratum V2 subprotocols
/// - Client session management
/// - Configuration options for server behavior
pub mod server;

/// Core service abstraction for Stratum V2 protocol implementations.
///
/// [`Sv2Service`] represents a long-running, stateful service that handles Stratum V2 protocol
/// events and produces outcomes. `Sv2Service` is designed for protocol-specific,
/// actor-like services that manage:
///
/// - **Connection lifecycles**: TCP connections, handshakes, and session management
/// - **Multi-protocol coordination**: Handling Mining, Job Declaration, and Template Distribution subprotocols
/// - **Inter-service communication**: Sibling service coordination for applications that act as both server and client
///
/// ## Event-Driven Architecture
///
/// Services process [`Event`](Self::Event)s which represent protocol messages, triggers, or internal
/// state changes. Each event produces an [`Outcome`](Self::Outcome) which may trigger additional
/// events, send responses to clients, or coordinate with sibling services.
pub trait Sv2Service {
    type Event;
    type Outcome;
    type ServiceError;
    type EventError;

    fn handle(
        &mut self,
        event: Self::Event,
    ) -> impl Future<Output = Result<Self::Outcome, Self::EventError>> + Send;

    fn start(&mut self) -> impl Future<Output = Result<(), Self::ServiceError>> + Send;
}

/// alias to abstract away [`codec_sv2::StandardEitherFrame`] and [`roles_logic_sv2::parsers::AnyMessage`]
pub type Sv2MessageFrame = StandardEitherFrame<AnyMessage<'static>>;

/// alias to abstract away [`codec_sv2::StandardSv2Frame`] and [`roles_logic_sv2::parsers::AnyMessage`]
pub type StandardSv2MessageFrame = StandardSv2Frame<AnyMessage<'static>>;

/// Sv2 Message IO as [`async_channel`] of [`Sv2MessageFrame`]
#[derive(Debug, Clone)]
pub struct Sv2MessageIo {
    // receiver channel of Sv2 Message Frames
    rx: Receiver<Sv2MessageFrame>,
    // sender channel of Sv2 Message Frames
    tx: Sender<Sv2MessageFrame>,
}

impl Sv2MessageIo {
    pub async fn send_message(
        &self,
        message: AnyMessage<'static>,
    ) -> Result<(), Sv2MessageIoError> {
        let frame: Sv2MessageFrame = message
            .clone()
            .try_into()
            .map_err(|_| Sv2MessageIoError::FrameError)
            .map(|sv2_frame: StandardSv2MessageFrame| sv2_frame.into())?;
        self.tx
            .send(frame)
            .await
            .map_err(|_| Sv2MessageIoError::SendError)?;
        Ok(())
    }

    pub async fn recv_message(&self) -> Result<AnyMessage<'static>, Sv2MessageIoError> {
        let frame = self
            .rx
            .recv()
            .await
            .map_err(|_| Sv2MessageIoError::RecvError)?;

        match frame {
            Frame::Sv2(mut frame) => {
                if let Some(header) = frame.get_header() {
                    let message_type = header.msg_type();
                    let mut payload = frame.payload().to_vec();
                    let message: Result<AnyMessage<'_>, _> =
                        (message_type, payload.as_mut_slice()).try_into();
                    match message {
                        Ok(message) => {
                            let message = Self::into_static(message);
                            Ok(message)
                        }
                        _ => Err(Sv2MessageIoError::FrameError),
                    }
                } else {
                    Err(Sv2MessageIoError::FrameError)
                }
            }
            Frame::HandShake(_) => Err(Sv2MessageIoError::FrameError),
        }
    }

    pub fn shutdown(&self) {
        self.tx.close();
        self.rx.close();
    }

    fn into_static(m: AnyMessage<'_>) -> AnyMessage<'static> {
        match m {
            AnyMessage::Mining(m) => AnyMessage::Mining(m.into_static()),
            AnyMessage::Common(m) => match m {
                CommonMessages::ChannelEndpointChanged(m) => {
                    AnyMessage::Common(CommonMessages::ChannelEndpointChanged(m.into_static()))
                }
                CommonMessages::SetupConnection(m) => {
                    AnyMessage::Common(CommonMessages::SetupConnection(m.into_static()))
                }
                CommonMessages::SetupConnectionError(m) => {
                    AnyMessage::Common(CommonMessages::SetupConnectionError(m.into_static()))
                }
                CommonMessages::SetupConnectionSuccess(m) => {
                    AnyMessage::Common(CommonMessages::SetupConnectionSuccess(m.into_static()))
                }
                CommonMessages::Reconnect(m) => {
                    AnyMessage::Common(CommonMessages::Reconnect(m.into_static()))
                }
            },
            AnyMessage::JobDeclaration(m) => match m {
                AllocateMiningJobToken(m) => {
                    AnyMessage::JobDeclaration(AllocateMiningJobToken(m.into_static()))
                }
                AllocateMiningJobTokenSuccess(m) => {
                    AnyMessage::JobDeclaration(AllocateMiningJobTokenSuccess(m.into_static()))
                }
                DeclareMiningJob(m) => {
                    AnyMessage::JobDeclaration(DeclareMiningJob(m.into_static()))
                }
                DeclareMiningJobError(m) => {
                    AnyMessage::JobDeclaration(DeclareMiningJobError(m.into_static()))
                }
                DeclareMiningJobSuccess(m) => {
                    AnyMessage::JobDeclaration(DeclareMiningJobSuccess(m.into_static()))
                }
                ProvideMissingTransactions(m) => {
                    AnyMessage::JobDeclaration(ProvideMissingTransactions(m.into_static()))
                }
                ProvideMissingTransactionsSuccess(m) => {
                    AnyMessage::JobDeclaration(ProvideMissingTransactionsSuccess(m.into_static()))
                }
                PushSolution(m) => AnyMessage::JobDeclaration(PushSolution(m.into_static())),
            },
            AnyMessage::TemplateDistribution(m) => match m {
                CoinbaseOutputConstraints(m) => {
                    AnyMessage::TemplateDistribution(CoinbaseOutputConstraints(m.into_static()))
                }
                TemplateDistribution::NewTemplate(m) => AnyMessage::TemplateDistribution(
                    TemplateDistribution::NewTemplate(m.into_static()),
                ),
                TemplateDistribution::RequestTransactionData(m) => {
                    AnyMessage::TemplateDistribution(TemplateDistribution::RequestTransactionData(
                        m.into_static(),
                    ))
                }
                TemplateDistribution::RequestTransactionDataError(m) => {
                    AnyMessage::TemplateDistribution(
                        TemplateDistribution::RequestTransactionDataError(m.into_static()),
                    )
                }
                TemplateDistribution::RequestTransactionDataSuccess(m) => {
                    AnyMessage::TemplateDistribution(
                        TemplateDistribution::RequestTransactionDataSuccess(m.into_static()),
                    )
                }
                TemplateDistribution::SetNewPrevHash(m) => AnyMessage::TemplateDistribution(
                    TemplateDistribution::SetNewPrevHash(m.into_static()),
                ),
                TemplateDistribution::SubmitSolution(m) => AnyMessage::TemplateDistribution(
                    TemplateDistribution::SubmitSolution(m.into_static()),
                ),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Sv2MessageIoError {
    FrameError,
    SendError,
    RecvError,
}
