use futures::pin_mut;
use tokio::sync::mpsc;

use xrc_device_manager::transport::btle::BtlePlugDeviceManagerBuilder;
use xrc_device_manager::transport::TransportManagerBuilder;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (event_sender, event_receiver) = mpsc::channel(256);

    let builder = BtlePlugDeviceManagerBuilder::default();

    let _manager = builder.finish(event_sender).unwrap();

    pin_mut!(event_receiver);
    while let Some(event) = event_receiver.recv().await {
        println!("Event: {:?}", event);
    }
}