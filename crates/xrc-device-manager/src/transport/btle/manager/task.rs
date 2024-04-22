use anyhow::Context;

use btleplug::{
  api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter},
  platform::{Adapter, Manager, Peripheral, PeripheralId},
};

use crate::{
  Result,
};
use tokio::sync::{
  oneshot, mpsc,
};
use futures::{future::FutureExt, StreamExt};
use tracing::{error, info};
use crate::transport::TransportManagerEvent;

pub enum BtlePlugManagerCommand {
  ScanStart(oneshot::Sender<Result<()>>),
  ScanStop(oneshot::Sender<Result<()>>),
}

pub(crate) struct BtlePlugDeviceManagerTask {
  command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
  event_sender: mpsc::Sender<TransportManagerEvent>,
}

impl BtlePlugDeviceManagerTask {
  pub fn new(
    command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
    event_sender: mpsc::Sender<TransportManagerEvent>,
  ) -> Self {
    Self {
      command_receiver,
      event_sender,
    }
  }

  pub async fn run(&mut self) -> Result<()> {
    info!("Starting BtlePlug Transport Task");

    let manager = Manager::new()
      .await
      .context("Unable to initialize btleplug manager")?;

    // todo: wait for adapter to be connected
    let central = manager
      .adapters()
      .await
      .context("Unable to fetch adapter list.")?
      .into_iter()
      .nth(0)
      .context("Unable to find adapters.")?;

    central.start_scan(ScanFilter::default()).await;

    info!("Adapter found: {:?}", central.adapter_info().await?);

    let mut events = central
      .events()
      .await
      .context("Unable to fetch adapter events.")?;

    loop {
      tokio::select! {
        event = events.next().fuse() => {
          match event {
            Some(event) => {
              info!("Received btleplug event: {:?}", event);
            },
            None => {
              error!("btleplug event stream ended");
              if let Err(err) = self.event_sender.send(TransportManagerEvent::ScanStopped).await {
                error!("Unable to send scanning finished event: {}", err);
              }
              break;
            }
          }
        }
      }
    }

    Ok(())
  }
}