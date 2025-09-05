use axum::extract::ws::Message;
use bh_sdk::v3::SdkMessage;
use getset::Getters;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::*;

use super::{HandlerBuilder, MessageHandler};
use crate::server::{HapticManagerCommand, HapticManagerEvent};

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[get = "pub"]
pub struct AppContext {
  workspace_id: String,
  api_key: String,
  version: Option<String>,
}

pub struct FeedbackHandlerBuilder {
  app_ctx: AppContext,
  command_sender: mpsc::Sender<HapticManagerCommand>,
  ws_sender: mpsc::UnboundedSender<Message>,
  cancellation_token: Option<CancellationToken>,
}

impl HandlerBuilder for FeedbackHandlerBuilder {
  type Handler = FeedbackHandler;
  type Context = AppContext;

  fn new(
    context: Self::Context,
    command_sender: mpsc::Sender<HapticManagerCommand>,
    ws_sender: mpsc::UnboundedSender<Message>,
  ) -> Self {
    Self {
      app_ctx: context,
      command_sender,
      ws_sender,
      cancellation_token: None,
    }
  }

  fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
    self.cancellation_token = Some(token);
    self
  }

  async fn build(self) -> anyhow::Result<Self::Handler> {
    let _cancellation_token = self.cancellation_token.unwrap_or_default();

    Ok(FeedbackHandler {
      app_ctx: self.app_ctx,
      command_sender: self.command_sender,
      ws_sender: self.ws_sender,
    })
  }
}

pub struct FeedbackHandler {
  app_ctx: AppContext,
  command_sender: mpsc::Sender<HapticManagerCommand>,
  ws_sender: mpsc::UnboundedSender<Message>,
}

impl MessageHandler for FeedbackHandler {
  type Context = AppContext;
  type Builder = FeedbackHandlerBuilder;

  #[instrument(skip(self))]
  async fn handle_connection_opened(&mut self) -> anyhow::Result<()> {
    info!("V3 WebSocket connection opened for workspace: {}", self.app_ctx.workspace_id());
    // TODO: Send welcome message or perform initial setup
    // Example:
    // let welcome_msg = ServerMessage::Welcome { workspace_id: self.app_ctx.workspace_id().clone() };
    // self.send_message(&welcome_msg).await?;
    Ok(())
  }

  #[instrument(skip(self, msg))]
  async fn handle_text_message(&mut self, msg: &str) -> anyhow::Result<()> {
    let sdk_msg: SdkMessage = serde_json::from_str(msg)
      .map_err(|e| anyhow::anyhow!("Failed to parse SDK message: {}", e))?;

    info!("V3 received SDK message: {:?}", sdk_msg);

    // TODO: Process the SDK message and send commands via self.command_sender
    // match sdk_msg {
    //   SdkMessage::StartFeedback { .. } => {
    //     let cmd = HapticManagerCommand::StartFeedback(...);
    //     self.command_sender.send(cmd).await?;
    //   }
    //   // ... other message types
    // }

    Ok(())
  }

  #[instrument(skip(self, data))]
  async fn handle_binary_message(&mut self, data: &[u8]) -> anyhow::Result<()> {
    info!("V3 received binary message of {} bytes", data.len());
    // TODO: Handle binary data if needed for v3
    Ok(())
  }

  #[instrument(skip(self))]
  async fn handle_close(&mut self) -> anyhow::Result<()> {
    info!("V3 WebSocket connection closing");
    Ok(())
  }

  #[instrument(skip(self, event))]
  async fn handle_haptic_event(&mut self, event: &HapticManagerEvent) -> anyhow::Result<()> {
    info!("V3 received haptic event: {:?}", event);
    // TODO: Convert HapticManagerEvent to v3 ServerMessage and send via ws_sender
    // Example:
    // let server_msg = self.convert_event_to_v3_message(event);
    // let json = serde_json::to_string(&server_msg)?;
    // self.ws_sender.send(Message::Text(json))?;
    Ok(())
  }
}

impl FeedbackHandler {
  #[instrument(skip(self, msg))]
  async fn send_message(&self, msg: &impl Serialize) -> anyhow::Result<()> {
    let json = serde_json::to_string(msg)?;
    self.ws_sender.send(Message::Text(json.into()))?;
    Ok(())
  }
}
