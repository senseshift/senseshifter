use dashmap::DashMap;
use futures::pin_mut;
use std::sync::Arc;
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

  let connected_devices = Arc::new(DashMap::<DeviceId, ConcurrentDevice>::new());

  loop {
    tokio::select! {
      Some(event) = event_receiver.recv() => {
        // info!("Got event: {:?}", event);

        match event {
          TransportManagerEvent::DeviceDiscovered {
            device,
          } => {
            if device.connectible() {
              match manager.connect_scanned(device.id()).await {
                Ok(_) => {
                  // info!("Connected to device: {:?}", device.descriptor());
                }
                Err(err) => {
                  error!("Error connecting to device: {:?}", err);
                }
              };
            }
          }
          TransportManagerEvent::DeviceConnected {
            device,
          } => {
            info!("Device connected: {:?}", device.descriptor());
            connected_devices.insert(device.id().clone(), device.clone());

            // sleep 30s to collect some properties
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

            let properties = device.properties().await;
            info!("Device properties: {:?}", properties);
          }
          TransportManagerEvent::DeviceDisconnected(id) => {
            info!("Device disconnected: {:?}", id);
            connected_devices.remove(&id);
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
