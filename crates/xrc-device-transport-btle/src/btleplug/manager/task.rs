use std::{
  sync::{
    Arc,
    atomic::{
      AtomicBool, Ordering,
    },
  }
};
use anyhow::{Context, Result};
use futures::{future::FutureExt, StreamExt};
use tokio::sync::{mpsc};
use dashmap::DashMap;

use tracing::{debug, error, info, instrument, trace, warn};

use btleplug::{
  api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter},
  platform::{Adapter, Manager, Peripheral, PeripheralId},
};

use xrc_transport::api::{Device, TransportManagerEvent};

use super::BtlePlugManagerCommand;
use crate::btleplug::{BtlePlugConnector, BtlePlugPeripheralInfo, BtlePlugProtocolSpecifier, PlatformBtlePlugConnector};

pub(super) struct BtlePlugManagerTask {
  command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
  event_sender: mpsc::Sender<TransportManagerEvent>,
  /// Determines if transport is ready to handle events.
  /// Shared between manager and task.
  adapter_connected: Arc<AtomicBool>,
  protocol_specifiers: Vec<Box<dyn BtlePlugProtocolSpecifier>>,
  known_addresses: DashMap<PeripheralId, Box<dyn Device>>,
}

impl BtlePlugManagerTask {
  pub fn new(
    command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
    event_sender: mpsc::Sender<TransportManagerEvent>,
    adapter_connected: Arc<AtomicBool>,
    protocol_specifiers: Vec<Box<dyn BtlePlugProtocolSpecifier>>,
  ) -> Self {
    Self {
      command_receiver,
      event_sender,
      adapter_connected,
      protocol_specifiers,
      known_addresses: DashMap::new(),
    }
  }

  #[tracing::instrument(skip(self))]
  pub async fn run(&mut self) -> Result<()> {
    info!("Starting BtlePlug Transport Task");

    let manager = Manager::new().await.context("Unable to initialize btleplug manager")?;

    // todo: wait for adapter to be connected
    let central = manager
      .adapters()
      .await.context("Unable to fetch adapter list.")?
      .into_iter()
      .nth(0).context("Unable to find adapters.")?;

    self.adapter_connected.store(true, Ordering::SeqCst);

    let mut events = central.events().await.context("Unable to fetch adapter events.")?;

    loop {
      tokio::select! {
        command = self.command_receiver.recv() => {
          match command {
            Some(command) => {
              self.handle_command(&central, command).await;
            },
            None => {
              info!("Bluetooth LE transport manager task shutting down");
              break;
            }
          }
        },
        event = events.next().fuse() => {
          match event {
            Some(event) => {
              self.handle_btle_event(&event, &central).await;
            },
            None => {
              error!("btleplug event stream ended");
              if let Err(err) = self.event_sender.send(TransportManagerEvent::ScanFinished).await {
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

  #[instrument(skip(self, adapter, command))]
  async fn handle_command(&mut self, adapter: &Adapter, command: BtlePlugManagerCommand) {
    match command {
      BtlePlugManagerCommand::ScanStart(sender) => {
        debug!("Starting Bluetooth LE scanning");
        let result = adapter.start_scan(ScanFilter::default()).await;
        let result = sender.send(result.map_err(|err| err.into()));
        if let Err(_err) = result {
          error!("Unable to send scanning started reply");
        }
      }
      BtlePlugManagerCommand::ScanStop(sender) => {
        debug!("Stopping Bluetooth LE scanning");
        let result = adapter.stop_scan().await;
        let result = sender.send(result.map_err(|err| err.into()));
        if let Err(_err) = result {
          error!("Unable to send scanning stopped reply");
        }
      }
    }
  }

  async fn handle_btle_event(&mut self, event: &CentralEvent, adapter: &Adapter) {
    match event {
      CentralEvent::DeviceDiscovered(peripheral_id) | CentralEvent::DeviceUpdated(peripheral_id) => {
        self.handle_peripheral(peripheral_id, adapter).await;
      }
      event => {
        trace!("Unhandled BTLE event: {:?}", event);
      }
    }
  }

  #[instrument(skip(self, adapter))]
  async fn handle_peripheral(&mut self, peripheral_id: &PeripheralId, adapter: &Adapter) {
    if self.known_addresses.contains_key(peripheral_id) {
      return;
    }

    let peripheral = match adapter.peripheral(peripheral_id).await {
      Ok(peripheral) => peripheral,
      Err(err) => {
        error!("Unable to fetch peripheral: {}", err);
        return;
      }
    };
    let peripheral_info = Self::get_peripheral_info(&peripheral).await;

    let connector = BtlePlugConnector::new(
      peripheral,
      peripheral_info,
    ) as PlatformBtlePlugConnector;

    let connector_clone = connector.clone();
    let device: Option<Box<dyn Device>> = self.protocol_specifiers
      .iter()
      .find_map(move |protocol_specifier| {
        match protocol_specifier.specify(connector_clone.clone()) {
          Ok(device) => device,
          Err(err) => {
            error!("Unable to specify device: {}", err);
            None
          }
        }
      });

    let device = match device {
      Some(device) => device,
      None => {
        return;
      }
    };

    self.known_addresses.insert(peripheral_id.clone(), device);

    let _ = self.event_sender.send(TransportManagerEvent::DeviceDiscovered {
      device_id: peripheral_id.to_string(),
    }).await.context("Unable to send device discovered event");
  }

  async fn get_peripheral_info(peripheral: &Peripheral) -> BtlePlugPeripheralInfo {
    let properties = match peripheral.properties().await {
      Ok(properties) => properties,
      Err(err) => {
        error!("Unable to fetch peripheral properties: {}", err);
        None
      }
    };

    let peripheral_info = BtlePlugPeripheralInfo {
      peripheral_id: peripheral.id().clone(),
      properties,
    };

    peripheral_info
  }
}