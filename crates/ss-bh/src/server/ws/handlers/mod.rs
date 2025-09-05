use axum::extract::ws::Message;
use serde::de::DeserializeOwned;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::server::{HapticManagerCommand, HapticManagerEvent};

// WebSocket message handler trait
pub trait MessageHandler: Send + Sync + 'static {
  type Context: DeserializeOwned + Send + Sync;
  type Builder: HandlerBuilder<Handler = Self, Context = Self::Context>;

  fn handle_connection_opened(&mut self) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
  fn handle_text_message(
    &mut self,
    msg: &str,
  ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
  fn handle_binary_message(
    &mut self,
    msg: &[u8],
  ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
  fn handle_close(&mut self) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
  fn handle_haptic_event(
    &mut self,
    event: &HapticManagerEvent,
  ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

// Handler builder trait
pub trait HandlerBuilder: Send + Sync {
  type Handler: MessageHandler;
  type Context;

  fn new(
    context: Self::Context,
    command_sender: mpsc::Sender<HapticManagerCommand>,
    ws_sender: mpsc::UnboundedSender<Message>,
  ) -> Self;

  fn with_cancellation_token(self, token: CancellationToken) -> Self;
  fn build(self) -> impl std::future::Future<Output = anyhow::Result<Self::Handler>> + Send;
}

#[cfg(feature = "v2")]
pub mod v2;

#[cfg(feature = "v3")]
pub mod v3;
