use anyhow::Context;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use btleplug::{
  api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter},
  platform::{Adapter, Manager, PeripheralId},
};
use dashmap::DashMap;

use derivative::Derivative;

use crate::api::*;
use crate::Result;
use futures::{future::FutureExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};

#[derive(Derivative)]
#[derivative(Debug)]
pub enum BtlePlugManagerCommand {
  ScanStart(#[derivative(Debug = "ignore")] oneshot::Sender<Result<()>>),
  ScanStop(#[derivative(Debug = "ignore")] oneshot::Sender<Result<()>>),
}

pub(crate) struct BtlePlugDeviceManagerTask<P: Peripheral> {
  command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
  event_sender: mpsc::Sender<TransportManagerEvent>,
  // TODO: Wait for https://github.com/rust-lang/rust/issues/65991 to stabilize
  // Later we can use trait `Arc<dyn BtlePlugDevice<P>>` instead of `Arc<BtlePlugDevice<P>>`
  // and upcast it to `Arc<dyn Device>`.
  // Example: https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=dafd9444190ec23b7558b7f16e10c63d
  discovered_devices: Arc<DashMap<DeviceId, Arc<BtlePlugDevice<P>>>>,
  connected_devices: Arc<DashMap<DeviceId, Arc<BtlePlugDevice<P>>>>,
  protocol_handlers: HashMap<String, Box<dyn BtlePlugProtocolSpecifier>>,
  adapter_ready: Arc<AtomicBool>,
  cancel_token: CancellationToken,
}

impl<P: Peripheral> BtlePlugDeviceManagerTask<P> {
  pub fn new(
    command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
    event_sender: mpsc::Sender<TransportManagerEvent>,
    discovered_devices: Arc<DashMap<DeviceId, Arc<BtlePlugDevice<P>>>>,
    connected_devices: Arc<DashMap<DeviceId, Arc<BtlePlugDevice<P>>>>,
    protocol_handlers: HashMap<String, Box<dyn BtlePlugProtocolSpecifier>>,
    adapter_ready: Arc<AtomicBool>,
    cancel_token: CancellationToken,
  ) -> Self {
    Self {
      command_receiver,
      event_sender,
      discovered_devices,
      connected_devices,
      protocol_handlers,
      adapter_ready,
      cancel_token,
    }
  }

  #[instrument(skip(self, central))]
  async fn handle_command(&self, command: BtlePlugManagerCommand, central: &Adapter) {
    match command {
      BtlePlugManagerCommand::ScanStart(sender) => {
        info!("Starting Bluetooth LE scanning");
        let result = central.start_scan(ScanFilter::default()).await;
        if sender.send(result.map_err(|err| err.into())).is_err() {
          error!("Unable to send scanning started reply");
        }
      }
      BtlePlugManagerCommand::ScanStop(sender) => {
        info!("Stopping Bluetooth LE scanning");
        let result = central.stop_scan().await;
        if sender.send(result.map_err(|err| err.into())).is_err() {
          error!("Unable to send scanning stopped reply");
        }
      }
    }
  }
}

impl BtlePlugDeviceManagerTask<btleplug::platform::Peripheral> {
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

    // Credit: https://github.com/buttplugio/buttplug/blob/master/buttplug/src/server/device/hardware/communication/btleplug/btleplug_adapter_task.rs
    #[cfg(target_os = "windows")]
    {
      use windows::Devices::Bluetooth::BluetoothAdapter;
      let adapter_result = BluetoothAdapter::GetDefaultAsync()
        .expect("If we're here, we got an adapter")
        .await;
      let adapter = adapter_result.expect("Considering infallible at this point");
      let device_id = adapter
        .DeviceId()
        .expect("Considering infallible at this point")
        .to_string();
      info!("Windows Bluetooth Adapter ID: {:?}", device_id);
      info!(
        "Windows Bluetooth Adapter Manufacturer: {}",
        device_manufacturer(device_id.as_str())
      );
    }

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
        },
        _ = self.cancel_token.cancelled().fuse() => {
          info!("Task cancelled");
          break;
        }
      }
    }

    Ok(())
  }

  async fn handle_btle_event(&self, event: CentralEvent, adapter: &Adapter) {
    match event {
      CentralEvent::DeviceDiscovered(peripheral_id)
      | CentralEvent::DeviceUpdated(peripheral_id) => {
        self.handle_peripheral_event(&peripheral_id, adapter).await;
      }
      CentralEvent::DeviceConnected(peripheral_id) => {
        let device_id = address_to_id(&peripheral_id.address());

        // todo: do not panic
        let device = self.discovered_devices.get(&device_id).unwrap();

        self.connected_devices.insert(device_id, device.clone());

        self
          .event_sender
          .send(TransportManagerEvent::DeviceConnected {
            device: device.clone() as ConcurrentDevice,
          })
          .await
          .unwrap_or_else(|err| {
            error!("Unable to send device connected event: {}", err);
          })
      }
      CentralEvent::DeviceDisconnected(peripheral_id) => {
        let device_id = address_to_id(&peripheral_id.address());

        self.connected_devices.remove(&device_id);
        self.discovered_devices.remove(&device_id);
        self
          .event_sender
          .send(TransportManagerEvent::DeviceDisconnected(device_id))
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
    let device_id = address_to_id(&peripheral_id.address());
    let existing = self.discovered_devices.get(&device_id);

    if let Some(existing) = existing {
      let device = existing.value().clone();
      self.handle_update_event(device).await;
      return;
    }

    let peripheral = match adapter.peripheral(peripheral_id).await {
      Ok(peripheral) => peripheral,
      Err(err) => {
        error!("Unable to fetch peripheral: {}", err);
        return;
      }
    };

    let mut device_internal = None;
    for (_, handler) in self.protocol_handlers.iter() {
      device_internal = handler
        .specify_protocol(peripheral.clone())
        .await
        .unwrap_or_else(|err| {
          error!("Unable to specify protocol: {}", err);
          None
        });

      if device_internal.is_some() {
        break;
      }
    }

    let device_internal = match device_internal {
      Some(device) => device,
      None => {
        return;
      }
    };

    let device = BtlePlugDevice::new(device_id, peripheral, device_internal);
    let device = Arc::new(device);

    device.handle_update_event().await.unwrap_or_else(|err| {
      error!("Error during initial device update: {}", err);
    });

    self
      .discovered_devices
      .insert(address_to_id(&peripheral_id.address()), device.clone());

    if let Err(err) = self
      .event_sender
      .send(TransportManagerEvent::DeviceDiscovered { device })
      .await
    {
      error!("Unable to send device discovered event: {}", err);
    }
  }

  async fn handle_update_event(&self, device: Arc<BtlePlugDevice<btleplug::platform::Peripheral>>) {
    device.handle_update_event().await.unwrap_or_else(|err| {
      error!("Error during internal device update: {}", err);
    });

    if let Err(err) = self
      .event_sender
      .send(TransportManagerEvent::DeviceUpdated {
        device: device.clone(),
      })
      .await
    {
      error!("Unable to send device updated event: {}", err);
    }
  }
}

/// Get the manufacturer of a Bluetooth device from its device ID.
///
/// Credit: https://github.com/buttplugio/buttplug/blob/master/buttplug/src/server/device/hardware/communication/btleplug/btleplug_adapter_task.rs
#[cfg(target_os = "windows")]
fn device_manufacturer(device_id: &str) -> &'static str {
  if device_id.contains("VID_0A12") {
    "Cambridge Silicon Radio (CSR)"
  } else if device_id.contains("VID_0A5C") {
    "Broadcom"
  } else if device_id.contains("VID_8087") {
    "Intel"
  } else if device_id.contains("VID_0BDA") {
    "RealTek"
  } else if device_id.contains("VID_0B05") {
    "Asus"
  } else if device_id.contains("VID_13D3") {
    "IMC"
  } else {
    "Unknown Manufacturer"
  }
}
