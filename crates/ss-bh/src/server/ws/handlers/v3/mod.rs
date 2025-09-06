use super::{HandlerBuilder, MessageHandler};
use crate::server::{HapticManagerCommand, HapticManagerEvent};
use axum::extract::ws::Message;
use bh_haptic_definitions::{HapticDefinitionsMessage, fetch_haptic_definitions};
use bh_sdk::v3::{SdkMessage, ServerEventListMessageItem, ServerMessage};
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
    match event {
      HapticManagerEvent::HapticEventsUpdated { namespace, events } => {
        if namespace != self.app_ctx.workspace_id() {
          return Ok(());
        }

        self
          .send_message(&ServerMessage::ServerEventNameList(
            events.iter().map(|e| e.name().clone()).collect::<Vec<_>>(),
          ))
          .await
          .map_err(|e| anyhow::anyhow!("Failed to send ServerEventNameList message: {}", e))?;

        self
          .send_message(&ServerMessage::ServerEventList(
            events
              .iter()
              .map(|e| ServerEventListMessageItem::new(e.name().clone(), e.event_time))
              .collect::<Vec<_>>(),
          ))
          .await
          .map_err(|e| anyhow::anyhow!("Failed to send ServerEventList message: {}", e))
      }
    }
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

        self.register_haptic_definitions(haptic_definitions).await?;

        self.send_message(&ServerMessage::ServerReady).await
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

        self.register_haptic_definitions(haptic_definitions).await?;

        self.send_message(&ServerMessage::ServerReady).await
      }
      SdkMessage::SdkStopAll => self
        .command_sender
        .send(HapticManagerCommand::StopAll {
          namespace: self.app_ctx.workspace_id().to_string(),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send StopAll command: {}", e)),
      SdkMessage::SdkPlayWithStartTime(msg) => self
        .command_sender
        .send(HapticManagerCommand::PlayEvent {
          namespace: self.app_ctx.workspace_id().to_string(),
          event_name: msg.event_name().to_string(),
          request_id: *msg.request_id(),
          start_millis: *msg.start_millis(),
          intensity: *msg.intensity(),
          duration: *msg.duration(),
          offset_x: *msg.offset_angle_x(),
          offset_y: *msg.offset_y(),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send PlayEvent command: {}", e)),
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::server::HapticEvent;
  use bh_sdk::v3::SdkPlayWithStartTimeMessage;
  use tokio::sync::mpsc;

  fn create_test_app_context() -> AppContext {
    AppContext {
      workspace_id: "test-workspace".to_string(),
      api_key: "test-api-key".to_string(),
      version: Some("1.0.0".to_string()),
    }
  }

  fn create_test_handler() -> (
    FeedbackHandler,
    mpsc::Receiver<HapticManagerCommand>,
    mpsc::UnboundedReceiver<Message>,
  ) {
    let app_ctx = create_test_app_context();
    let (command_tx, command_rx) = mpsc::channel(10);
    let (ws_tx, ws_rx) = mpsc::unbounded_channel();

    let handler = FeedbackHandler {
      app_ctx,
      command_sender: command_tx,
      ws_sender: ws_tx,
    };

    (handler, command_rx, ws_rx)
  }

  #[tokio::test]
  async fn test_send_message_serializes_and_sends() {
    let (handler, _command_rx, mut ws_rx) = create_test_handler();

    let result = handler.send_message(&ServerMessage::ServerReady).await;
    assert!(result.is_ok());

    let received = ws_rx.recv().await.unwrap();
    match received {
      Message::Text(text) => {
        let parsed: ServerMessage = serde_json::from_str(&text).unwrap();
        assert!(matches!(parsed, ServerMessage::ServerReady));
      }
      _ => panic!("Expected text message"),
    }
  }

  #[tokio::test]
  async fn test_handle_haptic_event_filters_by_namespace() {
    let (mut handler, _command_rx, mut ws_rx) = create_test_handler();

    let wrong_namespace_event = HapticManagerEvent::HapticEventsUpdated {
      namespace: "wrong-workspace".to_string(),
      events: vec![HapticEvent {
        name: "test-event".to_string(),
        event_time: 100,
      }],
    };

    let result = handler.handle_haptic_event(&wrong_namespace_event).await;
    assert!(result.is_ok());

    assert!(
      ws_rx.try_recv().is_err(),
      "Should not send messages for wrong namespace"
    );
  }

  #[tokio::test]
  async fn test_handle_haptic_event_sends_messages_for_correct_namespace() {
    let (mut handler, _command_rx, mut ws_rx) = create_test_handler();

    let events = vec![
      HapticEvent {
        name: "event1".to_string(),
        event_time: 100,
      },
      HapticEvent {
        name: "event2".to_string(),
        event_time: 200,
      },
    ];

    let correct_namespace_event = HapticManagerEvent::HapticEventsUpdated {
      namespace: "test-workspace".to_string(),
      events: events.clone(),
    };

    let result = handler.handle_haptic_event(&correct_namespace_event).await;
    assert!(result.is_ok());

    let msg1 = ws_rx.recv().await.unwrap();
    let msg2 = ws_rx.recv().await.unwrap();

    match msg1 {
      Message::Text(text) => {
        let parsed: ServerMessage = serde_json::from_str(&text).unwrap();
        match parsed {
          ServerMessage::ServerEventNameList(names) => {
            assert_eq!(names, vec!["event1", "event2"]);
          }
          _ => panic!("Expected ServerEventNameList"),
        }
      }
      _ => panic!("Expected text message"),
    }

    match msg2 {
      Message::Text(text) => {
        let parsed: ServerMessage = serde_json::from_str(&text).unwrap();
        match parsed {
          ServerMessage::ServerEventList(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(*items[0].event_name(), "event1");
            assert_eq!(*items[0].event_time(), 100);
            assert_eq!(*items[1].event_name(), "event2");
            assert_eq!(*items[1].event_time(), 200);
          }
          _ => panic!("Expected ServerEventList"),
        }
      }
      _ => panic!("Expected text message"),
    }
  }

  #[tokio::test]
  async fn test_handle_sdk_stop_all_sends_command() {
    let (mut handler, mut command_rx, _ws_rx) = create_test_handler();

    let result = handler.handle_sdk_message(&SdkMessage::SdkStopAll).await;
    assert!(result.is_ok());

    let command = command_rx.recv().await.unwrap();
    match command {
      HapticManagerCommand::StopAll { namespace } => {
        assert_eq!(namespace, "test-workspace");
      }
      _ => panic!("Expected StopAll command"),
    }
  }

  #[tokio::test]
  async fn test_register_haptic_definitions_sends_command() {
    let (handler, mut command_rx, _ws_rx) = create_test_handler();

    let definitions = HapticDefinitionsMessage::new(vec![])
      .with_id(Some("test-id".to_string()))
      .with_name(Some("test-name".to_string()));

    let result = handler
      .register_haptic_definitions(definitions.clone())
      .await;
    assert!(result.is_ok());

    let command = command_rx.recv().await.unwrap();
    match command {
      HapticManagerCommand::RegisterHapticDefinitions {
        namespace,
        definitions: received_defs,
      } => {
        assert_eq!(namespace, "test-workspace");
        assert_eq!(*received_defs, definitions);
      }
      _ => panic!("Expected RegisterHapticDefinitions command"),
    }
  }

  #[tokio::test]
  async fn test_handle_sdk_play_with_start_time_sends_command() {
    let (mut handler, mut command_rx, _ws_rx) = create_test_handler();

    let play_msg =
      SdkPlayWithStartTimeMessage::new("test-event".to_string(), 12345, 1000, 0.8, 0.5, -10.0, 5.0);

    let result = handler
      .handle_sdk_message(&SdkMessage::SdkPlayWithStartTime(play_msg))
      .await;
    assert!(result.is_ok());

    let command = command_rx.recv().await.unwrap();
    match command {
      HapticManagerCommand::PlayEvent {
        namespace,
        event_name,
        request_id,
        start_millis,
        intensity,
        duration,
        offset_x,
        offset_y,
      } => {
        assert_eq!(namespace, "test-workspace");
        assert_eq!(event_name, "test-event");
        assert_eq!(request_id, 12345);
        assert_eq!(start_millis, 1000);
        assert_eq!(intensity, 0.8);
        assert_eq!(duration, 0.5);
        assert_eq!(offset_x, -10.0);
        assert_eq!(offset_y, 5.0);
      }
      _ => panic!("Expected PlayEvent command"),
    }
  }

  #[tokio::test]
  async fn test_handle_text_message_with_valid_json() {
    let (mut handler, mut command_rx, _ws_rx) = create_test_handler();

    let json_msg = r#"{"type":"SdkStopAll"}"#;
    let result = handler.handle_text_message(json_msg).await;
    assert!(result.is_ok());

    let command = command_rx.recv().await.unwrap();
    assert!(matches!(command, HapticManagerCommand::StopAll { .. }));
  }

  #[tokio::test]
  async fn test_handle_text_message_with_invalid_json() {
    let (mut handler, _command_rx, _ws_rx) = create_test_handler();

    let invalid_json = r#"{"invalid": "message"}"#;
    let result = handler.handle_text_message(invalid_json).await;
    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("Failed to parse SDK message")
    );
  }
}
