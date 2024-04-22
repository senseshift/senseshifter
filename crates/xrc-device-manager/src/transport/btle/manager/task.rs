use std::sync::Arc;
use anyhow::Context;

use btleplug::{
  api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter},
  platform::{Adapter, Manager, Peripheral, PeripheralId},
};
use dashmap::DashMap;
use derivative::Derivative;

use crate::{
  Result,
};
use tokio::sync::{
  oneshot, mpsc,
};
use futures::{future::FutureExt, StreamExt};
use tracing::{error, info, instrument};
use crate::transport::btle::manager::peripheral::{BtlePlugPeripheral};
use crate::transport::TransportManagerEvent;

#[derive(Derivative)]
#[derivative(Debug)]
pub enum BtlePlugManagerCommand {
  ScanStart(
    #[derivative(Debug="ignore")]
    oneshot::Sender<Result<()>>
  ),
  ScanStop(
    #[derivative(Debug="ignore")]
    oneshot::Sender<Result<()>>
  ),
}

pub(crate) struct BtlePlugDeviceManagerTask {
  command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
  event_sender: mpsc::Sender<TransportManagerEvent>,
  scanned_peripherals: Arc<DashMap<PeripheralId, BtlePlugPeripheral>>,
}

impl BtlePlugDeviceManagerTask {
  pub fn new(
    command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
    event_sender: mpsc::Sender<TransportManagerEvent>,
    scanned_peripherals: Arc<DashMap<PeripheralId, BtlePlugPeripheral>>,
  ) -> Self {
    Self {
      command_receiver,
      event_sender,
      scanned_peripherals,
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
              self.handle_btle_event(event, &central).await;
            },
            None => {
              error!("btleplug event stream ended");
              if let Err(err) = self.event_sender.send(TransportManagerEvent::ScanStopped).await {
                error!("Unable to send scanning finished event: {}", err);
              }
              break;
            }
          }
        },
        command = self.command_receiver.recv().fuse() => {
          match command {
            Some(command) => {
              self.handle_command(command, &central).await;
            },
            None => {
              error!("Command channel closed");
              break;
            }
          }
        }
      }
    }

    Ok(())
  }

  #[instrument(skip(self, central))]
  async fn handle_command(&self, command: BtlePlugManagerCommand, central: &Adapter) {
    match command {
      BtlePlugManagerCommand::ScanStart(sender) => {
        info!("Starting Bluetooth LE scanning");
        let result = central.start_scan(ScanFilter::default()).await;
        let result = sender.send(result.map_err(|err| err.into()));
        if let Err(_err) = result {
          error!("Unable to send scanning started reply");
        }
      },
      BtlePlugManagerCommand::ScanStop(sender) => {
        info!("Stopping Bluetooth LE scanning");
        let result = central.stop_scan().await;
        let result = sender.send(result.map_err(|err| err.into()));
        if let Err(_err) = result {
          error!("Unable to send scanning stopped reply");
        }
      }
    }
  }

  async fn handle_btle_event(&self, event: CentralEvent, adapter: &Adapter) {
    match event {
      CentralEvent::DeviceDiscovered(peripheral_id) | CentralEvent::DeviceUpdated(peripheral_id) => {
        self.handle_peripheral_event(&peripheral_id, adapter).await;
      },
      _ => {}
    }
  }

  #[instrument(skip(self, adapter))]
  async fn handle_peripheral_event(&self, peripheral_id: &PeripheralId, adapter: &Adapter) {
    let entry = match self.scanned_peripherals.get_mut(peripheral_id) {
      Some(entry) => {
        let peripheral = entry.value();
        if let Err(err) = self.event_sender.send(TransportManagerEvent::DeviceUpdated(Box::new(peripheral.clone()))).await {
          error!("Unable to send device updated event: {}", err);
        }

        peripheral.clone()
      },
      None => {
        let peripheral = match adapter.peripheral(peripheral_id).await {
          Ok(peripheral) => peripheral,
          Err(err) => {
            error!("Unable to fetch peripheral: {}", err);
            return;
          }
        };

        let peripheral = BtlePlugPeripheral {
          peripheral,
        };

        self.scanned_peripherals.insert(peripheral_id.clone(), peripheral.clone());

        if let Err(err) = self.event_sender.send(TransportManagerEvent::DeviceDiscovered(Box::new(peripheral.clone()))).await {
          error!("Unable to send device discovered event: {}", err);
        }

        peripheral
      }
    };


  }
}