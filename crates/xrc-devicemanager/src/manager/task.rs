use std::{
  result::Result as StdResult,
};
use anyhow::Result;
use tracing::{debug, info};

use tokio_util::sync::CancellationToken;
use tokio::sync::{broadcast, mpsc};

use xrconnect_proto::devices::v1alpha1::{
  DeviceMessage,
};

pub(crate) struct DeviceManagerTask {
  pub(crate) event_sender: broadcast::Sender<DeviceMessage>,
  pub(crate) cancel_token: CancellationToken,
}

impl DeviceManagerTask {
  pub(crate) async fn run(&mut self) -> Result<()> {
    info!("Starting DeviceManagerTask...");

    loop {
      tokio::select! {
        _ = self.cancel_token.cancelled() => {
          break;
        }
      }
    }

    debug!("Exiting DeviceManagerTask...");

    Ok(())
  }
}