use std::{
  result::Result as StdResult,
};
use std::sync::Arc;
use anyhow::Result;
use log::trace;
use tracing::{debug, info, error};

use tokio_util::sync::CancellationToken;
use tokio::sync::{broadcast, mpsc};

use xrconnect_proto::devices::v1alpha1::{
  DeviceMessage,
  device_message::{
    ScanStarted,
    ScanStopped,
  },
};

use crate::transport::api::{TransportManager, TransportManagerEvent};
use super::DeviceManagerCommand;

pub(crate) struct DeviceManagerTask {
  pub(crate) transport_managers: Arc<Vec<Box<dyn TransportManager>>>,
  pub(crate) command_receiver: mpsc::Receiver<DeviceManagerCommand>,
  pub(crate) transport_event_receiver: mpsc::Receiver<TransportManagerEvent>,
  pub(crate) event_sender: broadcast::Sender<DeviceMessage>,
  pub(crate) cancel_token: CancellationToken,
}

impl DeviceManagerTask {
  #[tracing::instrument(skip(self))]
  async fn scan_start(&mut self) -> Result<()> {
    info!("No scan currently in progress, starting new scan.");

    match self.event_sender.send(ScanStarted {}.into()) {
      Ok(_) => {},
      Err(err) => {
        debug!("Failed to send scan started event: {}", err);
      }
    };

    Ok(())
  }

  #[tracing::instrument(skip(self))]
  async fn scan_stop(&mut self) -> Result<()> {
    debug!("Stopping scan...");

    match self.event_sender.send(ScanStopped {}.into()) {
      Ok(_) => {},
      Err(err) => {
        debug!("Failed to send scan stopped event: {}", err);
      }
    };

    Ok(())
  }

  async fn handle_command(&mut self, command: DeviceManagerCommand) -> Result<()> {
    match command {
      DeviceManagerCommand::ScanStart(response) => {
        let result = self.scan_start().await;
        response.send(result).map_err(|_| anyhow::anyhow!("Failed to send scan start response"))
      }
      DeviceManagerCommand::ScanStop(response) => {
        let result = self.scan_stop().await;
        response.send(result).map_err(|_| anyhow::anyhow!("Failed to send scan stop response"))
      }
    }
  }

  async fn handle_transport_event(&mut self, event: TransportManagerEvent) -> Result<()> {
    Ok(())
  }

  #[tracing::instrument(skip(self))]
  pub(crate) async fn run(&mut self) -> Result<()> {
    info!("Starting DeviceManagerTask...");

    loop {
      tokio::select! {
        Some(command) = self.command_receiver.recv() => {
          trace!("Received command: {:?}", command);
          if (self.handle_command(command).await.is_err()) {
            // error!("Error handling command: {:?}", command)
            error!("Error handling command")
          }
        }
        Some(event) = self.transport_event_receiver.recv() => {
          trace!("Received transport event: {:?}", event);
          if (self.handle_transport_event(event).await.is_err()) {
            // error!("Error handling transport event: {:?}", event)
            error!("Error handling transport event")
          }
        }
        _ = self.cancel_token.cancelled() => {
          break;
        }
      }
    }

    debug!("Exiting DeviceManagerTask...");

    Ok(())
  }
}