use super::{HandlerBuilder, MessageHandler};
use crate::server::{HapticManagerCommand, HapticManagerEvent};
use axum::extract::ws::Message;
use bh_haptic_definitions::{HapticDefinitionsMessage, fetch_haptic_definitions};
use bh_sdk::v3::SdkMessage;
use derive_more::Display;
use getset::Getters;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::*;

#[derive(Clone, Debug, Display, Getters, Serialize, Deserialize)]
#[display("AppContext {{ workspace_id={workspace_id}, api_key=*****, version={version:?} }}")]
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

  #[instrument(skip(self), fields(app = %self.app_ctx))]
  async fn handle_connection_opened(&mut self) -> anyhow::Result<()> {
    Ok(())
  }

  #[instrument(skip(self, msg), fields(app = %self.app_ctx))]
  async fn handle_text_message(&mut self, msg: &str) -> anyhow::Result<()> {
    let sdk_msg: SdkMessage = serde_json::from_str(msg)
      .map_err(|e| anyhow::anyhow!("Failed to parse SDK message: {}", e))?;

    self
      .handle_sdk_message(&sdk_msg)
      .await
      .map_err(|e| anyhow::anyhow!("Failed to handle SDK message {:?}: {}", sdk_msg, e))
  }

  #[instrument(skip(self, _data), fields(app = %self.app_ctx))]
  async fn handle_binary_message(&mut self, _data: &[u8]) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("Binary messages are not supported."))
  }

  #[instrument(skip(self), fields(app = %self.app_ctx))]
  async fn handle_close(&mut self) -> anyhow::Result<()> {
    Ok(())
  }

  #[instrument(skip(self, event), fields(app = %self.app_ctx))]
  async fn handle_haptic_event(&mut self, event: &HapticManagerEvent) -> anyhow::Result<()> {
    Ok(())
  }
}

impl FeedbackHandler {
  async fn send_message(&self, msg: &impl Serialize) -> anyhow::Result<()> {
    let json = serde_json::to_string(msg)?;
    self.ws_sender.send(Message::Text(json.into()))?;
    Ok(())
  }

  #[instrument(skip(self, msg), fields(app = %self.app_ctx))]
  async fn handle_sdk_message(&mut self, msg: &SdkMessage) -> anyhow::Result<()> {
    match msg {
      SdkMessage::SdkRequestAuth(msg) => {
        let haptic_definitions =
          fetch_haptic_definitions(msg.application_id(), msg.sdk_api_key()).await?;

        self.register_haptic_definitions(haptic_definitions).await
      }
      SdkMessage::SdkRequestAuthInit(msg) => {
        let haptic_definitions = match msg.haptic().message() {
          Some(defs) => defs.clone(),
          None => {
            fetch_haptic_definitions(
              msg.authentication().application_id(),
              msg.authentication().sdk_api_key(),
            )
            .await?
          }
        };

        self.register_haptic_definitions(haptic_definitions).await
      }
      SdkMessage::SdkStopAll => self
        .command_sender
        .send(HapticManagerCommand::StopAll {
          namespace: self.app_ctx.workspace_id().to_string(),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send StopAll command: {}", e)),
      _ => {
        Err(anyhow::anyhow!("Unsupported SDK message: {:?}", msg)) // todo
      }
    }
  }

  async fn register_haptic_definitions(
    &self,
    haptic_definitions: HapticDefinitionsMessage,
  ) -> anyhow::Result<()> {
    self
      .command_sender
      .send(HapticManagerCommand::RegisterHapticDefinitions {
        definitions: Box::new(haptic_definitions),
        namespace: self.app_ctx.workspace_id().to_string(),
      })
      .await
      .map_err(|e| anyhow::anyhow!("Failed to send RegisterHapticDefinitions command: {}", e))
  }
}
