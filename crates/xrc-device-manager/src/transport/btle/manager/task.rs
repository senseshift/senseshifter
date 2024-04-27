use anyhow::Context;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use btleplug::api::Peripheral;
use btleplug::{
  api::{Central, CentralEvent, Manager as _, ScanFilter},
  platform::{Adapter, Manager, PeripheralId},
};
use dashmap::DashMap;

use derivative::Derivative;

use crate::transport::btle::api::*;
use crate::transport::TransportManagerEvent;
use crate::Result;
use futures::{future::FutureExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, instrument};

#[derive(Derivative)]
#[derivative(Debug)]
pub enum BtlePlugManagerCommand {
  ScanStart(#[derivative(Debug = "ignore")] oneshot::Sender<Result<()>>),
  ScanStop(#[derivative(Debug = "ignore")] oneshot::Sender<Result<()>>),
}

pub(crate) struct BtlePlugDeviceManagerTask {
  command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
  event_sender: mpsc::Sender<TransportManagerEvent>,
  scanned_peripherals: Arc<DashMap<DeviceId, Box<dyn Device>>>,
  protocol_handlers: Arc<DashMap<String, Box<dyn BtlePlugProtocolHandler>>>,
  adapter_ready: Arc<AtomicBool>,
}

impl BtlePlugDeviceManagerTask {
  pub fn new(
    command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
    event_sender: mpsc::Sender<TransportManagerEvent>,
    scanned_peripherals: Arc<DashMap<DeviceId, Box<dyn Device>>>,
    protocol_handlers: Arc<DashMap<String, Box<dyn BtlePlugProtocolHandler>>>,
    adapter_connected: Arc<AtomicBool>,
  ) -> Self {
    Self {
      command_receiver,
      event_sender,
      scanned_peripherals,
      protocol_handlers,
      adapter_ready: adapter_connected,
    }
  }

  #[instrument(skip(self))]
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

    self.adapter_ready.store(true, Ordering::SeqCst);
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
      }
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
      CentralEvent::DeviceDiscovered(peripheral_id)
      | CentralEvent::DeviceUpdated(peripheral_id) => {
        self.handle_peripheral_event(&peripheral_id, adapter).await;
      }
      CentralEvent::DeviceConnected(peripheral_id) => self
        .event_sender
        .send(TransportManagerEvent::DeviceConnected {
          id: peripheral_id.to_string(),
          device: dyn_clone::clone(
            self
              .scanned_peripherals
              .get(&peripheral_id.to_string())
              .unwrap()
              .value(),
          ),
        })
        .await
        .unwrap_or_else(|err| {
          error!("Unable to send device connected event: {}", err);
        }),
      CentralEvent::DeviceDisconnected(peripheral_id) => {
        self.scanned_peripherals.remove(&peripheral_id.to_string());
        self
          .event_sender
          .send(TransportManagerEvent::DeviceDisconnected(
            peripheral_id.to_string(),
          ))
          .await
          .unwrap_or_else(|err| {
            error!("Unable to send device disconnected event: {}", err);
          });
      }
      _ => {}
    }
  }

  #[instrument(skip(self, adapter))]
  async fn handle_peripheral_event(&self, peripheral_id: &PeripheralId, adapter: &Adapter) {
    let existing = self.scanned_peripherals.get_mut(&peripheral_id.to_string());

    if let Some(mut existing) = existing {
      let peripheral = existing.value_mut();

      if let Err(err) = self
        .event_sender
        .send(TransportManagerEvent::DeviceUpdated {
          id: peripheral.id().to_string(),
          device: dyn_clone::clone(peripheral),
        })
        .await
      {
        error!("Unable to send device updated event: {}", err);
      }
      return;
    }

    let peripheral = match adapter.peripheral(peripheral_id).await {
      Ok(peripheral) => peripheral,
      Err(err) => {
        error!("Unable to fetch peripheral: {}", err);
        return;
      }
    };

    let peripheral_properties = match peripheral.properties().await {
      Ok(properties) => properties,
      Err(err) => {
        error!("Unable to fetch peripheral properties: {}", err);
        return;
      }
    };

    let mut candidate = None;
    for entry in self.protocol_handlers.iter() {
      let handler = entry.value();
      candidate = handler
        .specify_protocol(peripheral.clone(), peripheral_properties.clone())
        .unwrap_or_else(|err| {
          error!("Unable to specify protocol: {}", err);
          None
        });

      if candidate.is_some() {
        break;
      }
    }

    let candidate = match candidate {
      Some(candidate) => candidate,
      None => {
        return;
      }
    };

    self
      .scanned_peripherals
      .insert(peripheral_id.to_string(), dyn_clone::clone_box(&*candidate));

    if let Err(err) = self
      .event_sender
      .send(TransportManagerEvent::DeviceDiscovered {
        id: candidate.id(),
        device: candidate,
      })
      .await
    {
      error!("Unable to send device discovered event: {}", err);
    }
  }
}
