use ss_bh::server::ws::{BhWebsocketServerBuilder, BhWebsocketServerConfig};

use ss_bh::server::{HapticManagerCommand, HapticManagerEvent};
use std::path::PathBuf;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use tracing::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt()
    .pretty()
    .with_thread_names(true)
    .init();

  let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples")
    .join("simple_ws_server")
    .join("certs");

  let ws_config = BhWebsocketServerConfig::default()
    .with_tls_cert_path(Some(example_path.join("cert.pem")))
    .with_tls_key_path(Some(example_path.join("key.pem")));

  let cancellation_token = CancellationToken::new();

  let (command_sender, mut command_receiver) = mpsc::channel::<HapticManagerCommand>(10);
  let (event_sender, _event_receiver) = broadcast::channel::<HapticManagerEvent>(10);

  BhWebsocketServerBuilder::new(ws_config, command_sender, event_sender)
    .with_cancellation_token(Some(cancellation_token.clone()))
    .build()
    .await?;

  loop {
    tokio::select! {
      Some(command) = command_receiver.recv() => {
        info!("Received command: {:?}", command);
      },
      _ = tokio::signal::ctrl_c() => {
        println!("Received Ctrl+C, shutting down.");
        break;
      }
    }
  }

  cancellation_token.cancel();

  Ok(())
}
