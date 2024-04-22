mod task;

use futures::pin_mut;
use task::{BtlePlugDeviceManagerTask, BtlePlugManagerCommand};

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, warn};
use crate::Result;
use crate::transport::{TransportManager, TransportManagerBuilder, TransportManagerEvent};

#[derive(Default)]
pub struct BtlePlugDeviceManagerBuilder {

}

impl TransportManagerBuilder for BtlePlugDeviceManagerBuilder {
  fn finish(
    &self,
    event_sender: mpsc::Sender<TransportManagerEvent>,
  ) -> Result<Box<dyn TransportManager>> {
    let (task_command_sender, task_command_receiver) = mpsc::channel(256);

    let task = BtlePlugDeviceManagerTask::new(
      task_command_receiver,
      event_sender.clone(),
    );

    let task_handle: JoinHandle<_> = tokio::spawn(async move {
      pin_mut!(task);
      if let Err(err) = task.run().await {
        error!("BtlePlug Device Manager Task failed: {:?}", err);
      }
      warn!("BtlePlug Device Manager Task exited.");
    });

    Ok(Box::new(BtlePlugDeviceManager {
      task_handle,
      task_command_sender,
    }))
  }
}

pub struct BtlePlugDeviceManager {
  task_handle: JoinHandle<()>,
  task_command_sender: mpsc::Sender<BtlePlugManagerCommand>,
}

#[async_trait::async_trait]
impl TransportManager for BtlePlugDeviceManager {
  fn name(&self) -> &'static str {
    "BtlePlug"
  }
}