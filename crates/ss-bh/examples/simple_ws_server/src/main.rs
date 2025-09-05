use ss_bh::server::ws::{BhWebsocketServerBuilder, BhWebsocketServerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::init();

  let ws_config = BhWebsocketServerConfig::default();

  let sever_builder = BhWebsocketServerBuilder::new(ws_config);

  let _ = sever_builder.build().await?;

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
