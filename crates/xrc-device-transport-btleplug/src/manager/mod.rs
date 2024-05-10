mod task;

use std::collections::HashMap;

use btleplug::api::Peripheral;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dashmap::{DashMap, DashSet};
use futures::pin_mut;

use task::{BtlePlugDeviceManagerTask, BtlePlugManagerCommand};

use crate::api::*;
use crate::Result;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use tracing::{error, info, Instrument};

#[derive(Default)]
pub struct BtlePlugDeviceManagerBuilder {
  protocol_handlers: Vec<Box<dyn BtlePlugProtocolSpecifierBuilder>>,
}

impl BtlePlugDeviceManagerBuilder {
  pub fn protocol<T: BtlePlugProtocolSpecifierBuilder + 'static>(
    &mut self,
    builder: T,
  ) -> &mut Self {
    self.protocol_handlers.push(Box::new(builder));
    self
  }
}

impl TransportManagerBuilder for BtlePlugDeviceManagerBuilder {
  fn finish(
    &self,
    event_sender: mpsc::Sender<TransportManagerEvent>,
  ) -> Result<Box<dyn TransportManager>> {
    let (task_command_sender, task_command_receiver) = mpsc::channel(256);

    let adapter_ready = Arc::new(AtomicBool::new(false));

    // Create the protocol handlers
    let mut protocol_handlers = HashMap::new();
    for handler_builder in &self.protocol_handlers {
      let handler = handler_builder.finish();
      protocol_handlers.insert(handler.name().to_string(), handler);
    }

    let discovered_devices = Arc::new(DashMap::new());
    let connected_devices = Arc::new(DashMap::new());

    // Create the task
    let task_span = tracing::info_span!("BtlePlugDeviceManagerTask");
    let cancel_token = CancellationToken::new();
    let task = BtlePlugDeviceManagerTask::new(
      task_command_receiver,
      event_sender.clone(),
      discovered_devices.clone(),
      connected_devices.clone(),
      protocol_handlers,
      adapter_ready.clone(),
      cancel_token.clone(),
    );

    // Spawn the task
    let _join_token = tokio::spawn(
      async move {
        pin_mut!(task);
        if let Err(err) = task.run().await {
          error!("BtlePlug Transport Manager Task failed: {:?}", err);
        }
        info!("BtlePlug Transport Manager Task exited.");
      }
      .instrument(task_span),
    );

    Ok(Box::new(BtlePlugDeviceManager {
      task_command_sender,
      discovered_devices,
      adapter_ready,
      cancel_token,
      is_scanning: AtomicBool::new(false),
      connecting_devices: Arc::new(DashSet::new()),
    }))
  }
}

pub struct BtlePlugDeviceManager<P: Peripheral> {
  task_command_sender: mpsc::Sender<BtlePlugManagerCommand>,
  discovered_devices: Arc<DashMap<DeviceId, Arc<BtlePlugDevice<P>>>>,
  adapter_ready: Arc<AtomicBool>,
  cancel_token: CancellationToken,
  is_scanning: AtomicBool,
  connecting_devices: Arc<DashSet<DeviceId>>,
}

#[async_trait::async_trait]
impl<P: Peripheral + 'static> TransportManager for BtlePlugDeviceManager<P> {
  fn name(&self) -> &'static str {
    "BtlePlug"
  }

  fn ready(&self) -> bool {
    self.adapter_ready.load(Ordering::SeqCst)
  }

  fn is_scanning(&self) -> bool {
    self.is_scanning.load(Ordering::SeqCst)
  }

  async fn start_scanning(&mut self) -> Result<()> {
    self.is_scanning.store(true, Ordering::SeqCst);

    let (sender, receiver) = oneshot::channel();

    // Send the command to the task
    let _ = match self
      .task_command_sender
      .send(BtlePlugManagerCommand::ScanStart(sender))
      .await
    {
      Ok(_) => (),
      Err(err) => {
        self.is_scanning.store(false, Ordering::SeqCst);
        error!("Failed to send scan start command: {:?}", err);
        return Err(err.into());
      }
    };

    // wait for the result
    receiver.await.unwrap_or_else(|err| {
      self.is_scanning.store(false, Ordering::SeqCst);
      error!("Failed to receive scan start result: {:?}", err);
      Err(err.into())
    })
  }

  async fn stop_scanning(&mut self) -> Result<()> {
    self.is_scanning.store(false, Ordering::SeqCst);

    let (sender, receiver) = oneshot::channel();

    // Send the command to the task
    let _ = match self
      .task_command_sender
      .send(BtlePlugManagerCommand::ScanStop(sender))
      .await
    {
      Ok(_) => (),
      Err(err) => {
        self.is_scanning.store(true, Ordering::SeqCst);
        error!("Failed to send scan stop command: {:?}", err);
        return Err(err.into());
      }
    };

    // wait for the result
    receiver.await.unwrap_or_else(|err| {
      self.is_scanning.store(true, Ordering::SeqCst);
      error!("Failed to receive scan stop result: {:?}", err);
      Err(err.into())
    })
  }

  fn devices(&self) -> Result<Vec<ConcurrentDevice>> {
    let devices = self
      .discovered_devices
      .iter()
      .map(|v| v.value().clone() as ConcurrentDevice)
      .collect();

    Ok(devices)
  }

  fn get_device(&self, device_id: &DeviceId) -> Result<Option<ConcurrentDevice>> {
    let device = self
      .discovered_devices
      .get(device_id)
      .map(|v| v.value().clone() as ConcurrentDevice);

    Ok(device)
  }
}

impl<P: Peripheral> Drop for BtlePlugDeviceManager<P> {
  fn drop(&mut self) {
    self.cancel_token.cancel();
  }
}
