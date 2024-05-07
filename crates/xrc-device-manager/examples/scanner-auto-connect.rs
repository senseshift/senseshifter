use futures::{pin_mut, StreamExt};
use tracing::info;
use xrc_device_manager::manager::DeviceManagerBuilder;
use xrc_device_protocol_bhaptics::btleplug::BhapticsProtocolSpecifierBuilder;
use xrc_device_transport_btleplug::BtlePlugDeviceManagerBuilder;
use xrc_device_transport_serialport::manager::SerialPortTransportManagerBuilder;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let mut builder = DeviceManagerBuilder::default();

  {
    let mut btleplug_transport_builder = BtlePlugDeviceManagerBuilder::default();

    btleplug_transport_builder.protocol(BhapticsProtocolSpecifierBuilder::default());

    builder.transport(btleplug_transport_builder);
  }

  {
    let serialport_transport_builder = SerialPortTransportManagerBuilder::default();

    builder.transport(serialport_transport_builder);
  }

  let manager = builder.build().unwrap();
  manager.scan_start().await.unwrap();

  let event_stream = manager.event_stream();
  pin_mut!(event_stream);

  loop {
    tokio::select! {
      Some(event) = event_stream.next() => {
        info!("Got event: {:?}", event);
      },
      _ = tokio::signal::ctrl_c() => {
        info!("Received Ctrl-C, stopping...");
        break;
      }
    }
  }
}
