use futures::pin_mut;
use tokio::sync::mpsc;
use tracing::{error, info};

use xrc_device_manager::api::*;
use xrc_device_protocol_bhaptics::btleplug::BhapticsProtocolSpecifierBuilder;
use xrc_device_transport_btleplug::BtlePlugDeviceManagerBuilder;

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
          }
          | TransportManagerEvent::DeviceUpdated {
            device,
          } => {
            match manager.connect(device.descriptor().id()).await {
                Ok(_) => {
                  info!("Connected to device: {:?}", device.descriptor());
                }
                Err(err) => {
                  error!("Error connecting to device: {:?}", err);
                }
              };
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
