mod task;

use std::sync::Arc;

use dashmap::DashMap;
use futures::pin_mut;
use task::{BtlePlugDeviceManagerTask, BtlePlugManagerCommand};

use crate::transport::btle::api::*;
use crate::transport::{TransportManager, TransportManagerBuilder, TransportManagerEvent};
use crate::Result;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tracing::{error, warn};

#[derive(Default)]
pub struct BtlePlugDeviceManagerBuilder {
  protocol_handlers: Vec<Box<dyn BtlePlugProtocolHandlerBuilder>>,
}

impl BtlePlugDeviceManagerBuilder {
  pub fn with_protocol(
    mut self,
    protocol_handler: Box<dyn BtlePlugProtocolHandlerBuilder>,
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

    // Create the protocol handlers
    let protocol_handlers = Arc::new(DashMap::new());
    for handler_builder in &self.protocol_handlers {
      let handler = handler_builder.finish();
      protocol_handlers.insert(handler.name().to_string(), handler);
    }

    // Create the task
    let task = BtlePlugDeviceManagerTask::new(
      task_command_receiver,
      event_sender.clone(),
      Arc::new(DashMap::new()),
      protocol_handlers.clone(),
    );

    // Spawn the task
    tokio::spawn(async move {
      pin_mut!(task);
      if let Err(err) = task.run().await {
        error!("BtlePlug Device Manager Task failed: {:?}", err);
      }
      warn!("BtlePlug Device Manager Task exited.");
    });

    Ok(Box::new(BtlePlugDeviceManager {
      task_command_sender,
      protocol_handlers,
    }))
  }
}

pub struct BtlePlugDeviceManager {
  task_command_sender: mpsc::Sender<BtlePlugManagerCommand>,
  protocol_handlers: Arc<DashMap<String, Box<dyn BtlePlugProtocolHandler>>>,
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
}
