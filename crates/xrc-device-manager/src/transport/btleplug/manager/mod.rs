use anyhow::Result;
use futures::pin_mut;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::transport::api::{RawSerialDevice, TransportManager, TransportManagerBuilder, TransportManagerEvent};

mod task;
use task::BtlePlugManagerTask;

pub enum BtlePlugManagerCommand {
  ScanStart,
  ScanStop,
}

pub struct BtlePlugManagerBuilder {}

impl Default for BtlePlugManagerBuilder {
  fn default() -> Self {
    Self {}
  }
}

impl TransportManagerBuilder for BtlePlugManagerBuilder {
  fn finish(&self, event_sender: mpsc::Sender<TransportManagerEvent>) -> Result<Box<dyn TransportManager>> {
    let (command_sender, command_receiver) = mpsc::channel(256);
    let cancel_token = CancellationToken::new();

    let task = BtlePlugManagerTask {
      command_receiver,
      event_sender: event_sender.clone(),
      cancel_token: cancel_token.child_token(),
    };

    tokio::spawn(async move {
      pin_mut!(task);
      if let Err(err) = task.run().await {
        error!("Device manager task exited with error: {}", err);
      }
    });

    Ok(Box::new(BtlePlugManager {
      command_sender,
      event_sender,
      cancel_token,
    }))
  }
}

pub struct BtlePlugManager {
  command_sender: mpsc::Sender<BtlePlugManagerCommand>,
  event_sender: mpsc::Sender<TransportManagerEvent>,
  cancel_token: CancellationToken,
}

#[async_trait::async_trait]
impl TransportManager for BtlePlugManager {
  fn name(&self) -> &'static str {
    "btleplug"
  }

  async fn scan_start(&mut self) -> anyhow::Result<()> {
    Ok(self.command_sender.send(BtlePlugManagerCommand::ScanStart).await?) // todo: oneshot return?
  }

  async fn scan_stop(&mut self) -> anyhow::Result<()> {
    Ok(self.command_sender.send(BtlePlugManagerCommand::ScanStop).await?) // todo: oneshot return?
  }

  fn is_scanning(&self) -> bool {
    todo!()
  }

  fn ready(&self) -> bool {
    todo!()
  }
}