mod task;

use std::collections::HashMap;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use dashmap::DashMap;

use task::{BtlePlugDeviceManagerTask, BtlePlugManagerCommand};

use crate::api::*;
use crate::Result;
use tokio::sync::{mpsc, oneshot};

use tracing::{error, instrument, warn};

#[derive(Default)]
pub struct BtlePlugDeviceManagerBuilder {
  protocol_handlers: Vec<Box<dyn BtlePlugProtocolSpecifierBuilder>>,
}

impl BtlePlugDeviceManagerBuilder {
  pub fn with_protocol(
    mut self,
    protocol_handler: Box<dyn BtlePlugProtocolSpecifierBuilder>,
  ) -> Self {
    self.protocol_handlers.push(protocol_handler);
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

    // Create the task
    let mut task = BtlePlugDeviceManagerTask::new(
      task_command_receiver,
      event_sender.clone(),
      discovered_devices.clone(),
      protocol_handlers,
      adapter_ready.clone(),
    );

    // Spawn the task
    tokio::spawn(async move {
      if let Err(err) = task.run().await {
        error!("BtlePlug Device Manager Task failed: {:?}", err);
      }
      warn!("BtlePlug Device Manager Task exited.");
    });

    Ok(Box::new(BtlePlugDeviceManager {
      task_command_sender,
      discovered_devices,
      adapter_ready,
    }))
  }
}

pub struct BtlePlugDeviceManager {
  task_command_sender: mpsc::Sender<BtlePlugManagerCommand>,
  discovered_devices: Arc<DashMap<DeviceId, Box<dyn Device>>>,
  adapter_ready: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl TransportManager for BtlePlugDeviceManager {
  fn name(&self) -> &'static str {
    "BtlePlug"
  }

  async fn start_scanning(&self) -> Result<()> {
    let (sender, receiver) = oneshot::channel();

    // Send the command to the task
    let _ = match self
      .task_command_sender
      .send(BtlePlugManagerCommand::ScanStart(sender))
      .await
    {
      Ok(_) => (),
      Err(err) => {
        error!("Failed to send scan start command: {:?}", err);
        return Err(err.into());
      }
    };

    // wait for the result
    receiver.await.unwrap_or_else(|err| {
      error!("Failed to receive scan start result: {:?}", err);
      Err(err.into())
    })
  }

  async fn stop_scanning(&self) -> Result<()> {
    let (sender, receiver) = oneshot::channel();

    // Send the command to the task
    let _ = match self
      .task_command_sender
      .send(BtlePlugManagerCommand::ScanStop(sender))
      .await
    {
      Ok(_) => (),
      Err(err) => {
        error!("Failed to send scan stop command: {:?}", err);
        return Err(err.into());
      }
    };

    // wait for the result
    receiver.await.unwrap_or_else(|err| {
      error!("Failed to receive scan stop result: {:?}", err);
      Err(err.into())
    })
  }

  #[instrument(skip(self))]
  async fn connect(&self, device_id: &DeviceId) -> Result<()> {
    let (sender, receiver) = oneshot::channel();

    // Send the command to the task
    let _ = match self
      .task_command_sender
      .send(BtlePlugManagerCommand::ConnectDevice(
        device_id.clone(),
        sender,
      ))
      .await
    {
      Ok(_) => (),
      Err(err) => {
        error!("Failed to send connect command: {:?}", err);
        return Err(err.into());
      }
    };

    // wait for the result
    receiver.await.unwrap_or_else(|err| {
      error!("Failed to receive connect result: {:?}", err);
      Err(err.into())
    })
  }
}
