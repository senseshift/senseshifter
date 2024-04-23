use futures::pin_mut;
use tokio::sync::mpsc;

use xrc_device_manager::transport::btle::protocol::bhaptics::BhapticsProtocolHandlerBuilder;
use xrc_device_manager::transport::btle::BtlePlugDeviceManagerBuilder;
use xrc_device_manager::transport::TransportManagerBuilder;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let (event_sender, event_receiver) = mpsc::channel(256);

  let builder = BtlePlugDeviceManagerBuilder::default()
    .with_protocol(Box::<BhapticsProtocolHandlerBuilder>::default());

  let manager = builder.finish(event_sender).unwrap();

  manager
    .start_scanning()
    .await
    .expect("Failed to start scanning");

  pin_mut!(event_receiver);
  while let Some(event) = event_receiver.recv().await {
    println!("Event: {:?}", event);
  }
}
