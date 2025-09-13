#![cfg(all(feature = "v3", feature = "serde", feature = "ws"))]

use std::net::SocketAddr;
use std::time::Duration;

use bh_sdk::v3::{SdkMessage, ServerMessage};
use ss_bh::server::ws::{BhWebsocketServerBuilder, BhWebsocketServerConfig};
use ss_bh::server::{HapticEvent, HapticManagerCommand, HapticManagerEvent};

use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::time::timeout;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;
use tracing::*;

// Helper function to read the next text message, skipping ping/pong messages
async fn read_next_text_message(
  receiver: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
  timeout_duration: Duration,
) -> anyhow::Result<String> {
  loop {
    let msg = timeout(timeout_duration, receiver.next())
      .await
      .map_err(|_| anyhow::anyhow!("Timeout waiting for message"))?
      .ok_or_else(|| anyhow::anyhow!("Stream ended"))?
      .map_err(|e| anyhow::anyhow!("WebSocket error: {}", e))?;

    match msg {
      Message::Text(text) => return Ok(text.to_string()),
      Message::Ping(_) | Message::Pong(_) => {
        debug!("Skipping ping/pong message");
        continue;
      }
      other => return Err(anyhow::anyhow!("Unexpected message type: {:?}", other)),
    }
  }
}

#[tokio::test]
async fn test_websocket_server_lifecycle_and_message_exchange() {
  tracing_subscriber::fmt()
    .with_test_writer()
    .with_max_level(tracing::Level::DEBUG)
    .try_init()
    .ok();

  // Setup server channels
  let (command_tx, mut command_rx) = mpsc::channel::<HapticManagerCommand>(10);
  let (event_tx, _event_rx) = broadcast::channel::<HapticManagerEvent>(10);
  let cancellation_token = CancellationToken::new();

  // Use a specific port for reliable testing
  let server_addr: SocketAddr = "127.0.0.1:15883".parse().unwrap();

  // Create config with only HTTP (no TLS certificates)
  let mut ws_config = BhWebsocketServerConfig::default().with_listen(Some(server_addr));

  #[cfg(feature = "tls")]
  {
    ws_config = ws_config
      .with_listen_tls(None)
      .with_tls_cert_path(None) // Disable TLS for tests
      .with_tls_key_path(None);
  }

  info!("Starting test server on {}...", server_addr);

  // Start server in background
  let server_token = cancellation_token.clone();
  let event_tx_clone = event_tx.clone();
  let server_handle = tokio::spawn(async move {
    BhWebsocketServerBuilder::new(ws_config, command_tx, event_tx_clone)
      .with_cancellation_token(Some(server_token))
      .build()
      .await
  });

  // Give server time to start
  tokio::time::sleep(Duration::from_millis(500)).await;

  // Test 1: Connect WebSocket client
  let ws_url = format!(
    "ws://{}/v3/feedback?workspace_id=test-workspace&api_key=test-key",
    server_addr
  );
  info!("Connecting to: {}", ws_url);

  let (ws_stream, _response) = timeout(Duration::from_secs(5), connect_async(&ws_url))
    .await
    .expect("Connection timeout")
    .expect("Failed to connect to WebSocket");

  info!("WebSocket connection established");

  let (mut ws_sender, mut ws_receiver) = ws_stream.split();

  // Test 2: Send SDK message and verify command is received
  let sdk_message = SdkMessage::SdkStopAll;
  let json_message = serde_json::to_string(&sdk_message).unwrap();

  info!("Sending message: {}", json_message);
  ws_sender
    .send(Message::Text(json_message.into()))
    .await
    .expect("Failed to send message");

  // Verify command was received on the server side
  let received_command = timeout(Duration::from_secs(2), command_rx.recv())
    .await
    .expect("Command timeout")
    .expect("No command received");

  match received_command {
    HapticManagerCommand::StopAll { namespace } => {
      assert_eq!(namespace, "test-workspace");
      info!("✅ Received expected StopAll command with correct namespace");
    }
    other => panic!("Expected StopAll command, got {:?}", other),
  }

  // Test 3: Send a more complex message (PlayEvent)
  let play_message =
    SdkMessage::SdkPlayWithStartTime(bh_sdk::v3::SdkPlayWithStartTimeMessage::new(
      "test-event".to_string(),
      12345,
      1000,
      0.8,
      0.5,
      -10.0,
      5.0,
    ));
  let play_json = serde_json::to_string(&play_message).unwrap();

  info!("Sending PlayEvent message: {}", play_json);
  ws_sender
    .send(Message::Text(play_json.into()))
    .await
    .expect("Failed to send play message");

  // Verify PlayEvent command
  let play_command = timeout(Duration::from_secs(2), command_rx.recv())
    .await
    .expect("Play command timeout")
    .expect("No play command received");

  match play_command {
    HapticManagerCommand::PlayEvent {
      namespace,
      event_name,
      request_id,
      start_millis,
      intensity,
      duration,
      offset_angle_x,
      offset_y,
    } => {
      assert_eq!(namespace, "test-workspace");
      assert_eq!(event_name, "test-event");
      assert_eq!(request_id, 12345);
      assert_eq!(start_millis, 1000);
      assert_eq!(intensity, 0.8);
      assert_eq!(duration, 0.5);
      assert_eq!(offset_angle_x, -10.0);
      assert_eq!(offset_y, 5.0);
      info!("✅ Received expected PlayEvent command with correct parameters");
    }
    other => panic!("Expected PlayEvent command, got {:?}", other),
  }

  // Test 4: Verify server can send messages back to client
  // Simulate sending a haptic event update
  let haptic_events = vec![
    HapticEvent::new("test-event-1".to_string(), 100),
    HapticEvent::new("test-event-2".to_string(), 200),
  ];

  let haptic_event = HapticManagerEvent::HapticEventsUpdated {
    namespace: "test-workspace".to_string(),
    events: haptic_events,
  };

  // Send event (this should trigger server to send messages to client)
  event_tx.send(haptic_event).unwrap();

  // Receive and verify server messages
  let text1 = read_next_text_message(&mut ws_receiver, Duration::from_secs(2))
    .await
    .expect("Failed to read first server message");

  let text2 = read_next_text_message(&mut ws_receiver, Duration::from_secs(2))
    .await
    .expect("Failed to read second server message");

  // Parse and verify messages
  let server_msg1: ServerMessage = serde_json::from_str(&text1).unwrap();
  match server_msg1 {
    ServerMessage::ServerEventNameList(names) => {
      assert_eq!(names, vec!["test-event-1", "test-event-2"]);
      info!("✅ Received expected ServerEventNameList");
    }
    other => panic!("Expected ServerEventNameList, got {:?}", other),
  }

  let server_msg2: ServerMessage = serde_json::from_str(&text2).unwrap();
  match server_msg2 {
    ServerMessage::ServerEventList(items) => {
      assert_eq!(items.len(), 2);
      assert_eq!(*items[0].event_name(), "test-event-1");
      assert_eq!(*items[0].event_time(), 100);
      assert_eq!(*items[1].event_name(), "test-event-2");
      assert_eq!(*items[1].event_time(), 200);
      info!("✅ Received expected ServerEventList with correct data");
    }
    other => panic!("Expected ServerEventList, got {:?}", other),
  }

  // Test 5: Graceful cleanup
  info!("Starting graceful shutdown");
  cancellation_token.cancel();

  // Give server time to shut down
  let server_result = timeout(Duration::from_secs(5), server_handle)
    .await
    .expect("Server shutdown timeout")
    .expect("Server task panicked");

  assert!(server_result.is_ok(), "Server should shut down gracefully");
  info!("✅ Server shut down gracefully");

  info!("🎉 All WebSocket integration tests passed!");
}

#[tokio::test]
async fn test_websocket_invalid_message_handling() {
  tracing_subscriber::fmt()
    .with_test_writer()
    .with_max_level(tracing::Level::DEBUG)
    .try_init()
    .ok();

  // Setup server
  let (command_tx, mut command_rx) = mpsc::channel::<HapticManagerCommand>(10);
  let (event_tx, _event_rx) = broadcast::channel::<HapticManagerEvent>(10);
  let cancellation_token = CancellationToken::new();

  // Use a different port for the second test
  let server_addr: SocketAddr = "127.0.0.1:15884".parse().unwrap();

  let mut ws_config = BhWebsocketServerConfig::default().with_listen(Some(server_addr));

  #[cfg(feature = "tls")]
  {
    ws_config = ws_config
      .with_listen_tls(None)
      .with_tls_cert_path(None)
      .with_tls_key_path(None);
  }

  let server_token = cancellation_token.clone();
  let server_handle = tokio::spawn(async move {
    BhWebsocketServerBuilder::new(ws_config, command_tx, event_tx)
      .with_cancellation_token(Some(server_token))
      .build()
      .await
  });

  tokio::time::sleep(Duration::from_millis(500)).await;

  // Connect client
  let ws_url = format!(
    "ws://{}/v3/feedback?workspace_id=test-workspace&api_key=test-key",
    server_addr
  );
  let (ws_stream, _response) = connect_async(&ws_url).await.unwrap();
  let (mut ws_sender, _ws_receiver) = ws_stream.split();

  // Send an invalid JSON message
  let invalid_json = r#"{"invalid": "message format"}"#;
  ws_sender
    .send(Message::Text(invalid_json.to_string().into()))
    .await
    .expect("Failed to send invalid message");

  // Verify no command is received (invalid messages should be handled gracefully)
  let command_result = timeout(Duration::from_millis(500), command_rx.recv()).await;
  assert!(
    command_result.is_err(),
    "Should not receive command for invalid message"
  );

  info!("✅ Invalid message handled gracefully - no command generated");

  // Cleanup
  cancellation_token.cancel();
  let _ = timeout(Duration::from_secs(2), server_handle).await;

  info!("🎉 Invalid message handling test passed!");
}
