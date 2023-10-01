use anyhow::Result;
use futures::{pin_mut, StreamExt};
use tonic::transport::Server;
use tracing::info;

use xrc_device_manager::DeviceManager;
use xrconnect_proto::devices::v1alpha1::device_manager_server::DeviceManagerServer;

use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt::init();

  let addr = "[::1]:50051".parse()?;

  let manager = DeviceManager::default();

  let event_stream = manager.event_stream();
  tokio::spawn(async move {
    pin_mut!(event_stream);
    while let Some(event) = event_stream.next().await {
      info!("Event: {:?}", event);
    }
  });

  manager.scan_start().await?;

  Server::builder()
    .add_service(DeviceManagerServer::new(manager))
    .serve(addr)
    .await?;

  Ok(())
}
