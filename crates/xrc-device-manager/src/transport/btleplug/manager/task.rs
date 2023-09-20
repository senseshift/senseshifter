use anyhow::Result;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::BtlePlugManagerCommand;

pub(super) struct BtlePlugManagerTask {
  pub command_receiver: mpsc::Receiver<BtlePlugManagerCommand>,
  pub event_sender: mpsc::Sender<super::TransportManagerEvent>,
  pub cancel_token: CancellationToken,
}

impl BtlePlugManagerTask {
  pub async fn run(&mut self) -> Result<()> {
    Ok(())
  }
}