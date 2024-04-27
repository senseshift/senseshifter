use futures::pin_mut;
use tokio::sync::mpsc;
use tracing::{error, info};

use xrc_device_manager::api::*;
use xrc_device_transport_btleplug::api::*;
use xrc_device_transport_btleplug::BtlePlugDeviceManagerBuilder;
use xrc_protocol_bhaptics::btleplug::BhapticsProtocolSpecifierBuilder;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let (event_sender, event_receiver) = mpsc::channel(256);

  let builder = BtlePlugDeviceManagerBuilder::default()
    .with_protocol(Box::<BhapticsProtocolSpecifierBuilder>::default());

  let manager = builder.finish(event_sender).unwrap();

  manager
    .start_scanning()
    .await
    .expect("Failed to start scanning");

  pin_mut!(event_receiver);

  loop {
    tokio::select! {
      Some(event) = event_receiver.recv() => {
        // info!("Got event: {:?}", event);

        match event {
          TransportManagerEvent::DeviceDiscovered {
            device,
            id: device_id,
          }
          | TransportManagerEvent::DeviceUpdated {
            device,
            id: device_id,
          } => {
            if device.connectible() && !device.connected() {
              match manager.connect(&device_id).await {
                Ok(_) => {
                  info!("Connected to device: {:?}", device.name());
                }
                Err(err) => {
                  error!("Error connecting to device: {:?}", err);
                }
              };
            }
          }
          _ => {}
        }
      },
      _ = tokio::signal::ctrl_c() => {
        info!("Received Ctrl-C, stopping scanning.");
        manager.stop_scanning().await.expect("Failed to stop scanning");
        break;
      }
    }
  }
}
