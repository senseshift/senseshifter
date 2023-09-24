use std::{
  result::Result as StdResult,
};
use std::future::Future;
use std::sync::Arc;
use anyhow::Result;
use futures_util::future::join_all;
use tracing::{trace, debug, info, error};

use tokio_util::sync::CancellationToken;
use tokio::sync::{broadcast, mpsc};

use xrc_transport::api::{TransportManager, TransportManagerEvent};
use super::{
  DeviceManagerCommand, DeviceManagerEvent,
};

pub(crate) struct DeviceManagerTask {
  pub(crate) transport_managers: Vec<Box<dyn TransportManager>>,
  pub(crate) command_receiver: mpsc::Receiver<DeviceManagerCommand>,
  pub(crate) transport_event_receiver: mpsc::Receiver<TransportManagerEvent>,
  pub(crate) event_sender: broadcast::Sender<DeviceManagerEvent>,
  pub(crate) cancel_token: CancellationToken,
}

impl DeviceManagerTask {
  fn is_scanning(&self) -> bool {
    self.transport_managers.iter().any(|manager| manager.is_scanning())
  }

  #[tracing::instrument(skip(self))]
  async fn scan_start(&mut self) -> Result<()> {
    if self.is_scanning() {
      debug!("Scan already in progress, ignoring scan start request.");
      return Ok(());
    }

    info!("No scan currently in progress, starting new scan.");

    let futures: Vec<_> = self.transport_managers
      .iter_mut()
      .map(|manager| manager.scan_start())
      .collect();

    join_all(futures).await
      .into_iter()
      .filter_map(|result| result.err())
      .for_each(|err| {
        error!("Error starting scan: {}", err);
      });

    match self.event_sender.send(DeviceManagerEvent::ScanStarted) {
      Ok(_) => {}
      Err(err) => {
        debug!("Failed to send scan started event: {}", err);
      }
    };

    Ok(())
  }

  #[tracing::instrument(skip(self))]
  async fn scan_stop(&mut self) -> Result<()> {
    if !self.is_scanning() {
      debug!("No scan currently in progress, ignoring scan stop request.");
      return Ok(());
    }

    info!("Scan currently in progress, stopping scan.");

    let futures: Vec<_> = self.transport_managers
      .iter_mut()
      .map(|manager| manager.scan_stop())
      .collect();

    join_all(futures).await
      .into_iter()
      .filter_map(|result| result.err())
      .for_each(|err| {
        error!("Error stopping scan: {}", err);
      });

    match self.event_sender.send(DeviceManagerEvent::ScanStopped) {
      Ok(_) => {}
      Err(err) => {
        debug!("Failed to send scan stopped event: {}", err);
      }
    };

    Ok(())
  }

  #[tracing::instrument(skip(self))]
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
    match event {
      TransportManagerEvent::ScanFinished => {
        // Some transports are still scanning, so don't send the scan stopped event yet.
        if self.is_scanning() {
          return Ok(());
        }

        self.event_sender.send(DeviceManagerEvent::ScanStopped)?;
      }
      TransportManagerEvent::DeviceDiscovered { device_id, device } => {
        self.event_sender.send(DeviceManagerEvent::DeviceDiscovered { device_id, device })?;
      }
      TransportManagerEvent::DeviceUpdated { device_id, device } => {
        self.event_sender.send(DeviceManagerEvent::DeviceUpdated { device_id, device })?;
      }
    }
    Ok(())
  }

  #[tracing::instrument(skip(self))]
  pub(crate) async fn run(&mut self) -> Result<()> {
    info!("Starting DeviceManagerTask...");

    loop {
      tokio::select! {
        Some(command) = self.command_receiver.recv() => {
          trace!("Received command: {:?}", command);
          if let Err(err) = self.handle_command(command).await {
            error!("Error handling command: {}", err);
          }
        }
        Some(event) = self.transport_event_receiver.recv() => {
          trace!("Received transport event: {:?}", event);
          if let Err(err) = self.handle_transport_event(event).await {
            error!("Error handling transport event: {}", err);
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