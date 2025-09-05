use ss_bh::server::ws::{BhWebsocketServerBuilder, BhWebsocketServerConfig};

use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

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

  BhWebsocketServerBuilder::new(ws_config)
    .with_cancellation_token(Some(cancellation_token))
    .build()
    .await?;

  loop {
    tokio::select! {
      _ = tokio::signal::ctrl_c() => {
        println!("Received Ctrl+C, shutting down.");
        break;
      }
    }
  }

  Ok(())
}
