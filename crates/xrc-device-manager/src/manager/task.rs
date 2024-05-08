use crate::api::*;
use crate::manager::DeviceManagerCommand;
use crate::Result;
use dashmap::DashMap;
use futures_util::future::join_all;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};

pub(crate) struct DeviceManagerTask {
  cancel_token: CancellationToken,
  transport_managers: Vec<Box<dyn TransportManager>>,
  transport_event_receiver: mpsc::Receiver<TransportManagerEvent>,
  event_sender: broadcast::Sender<DeviceManagerEvent>,
  task_command_receiver: mpsc::Receiver<DeviceManagerCommand>,
  discovered_devices: Arc<DashMap<DeviceId, ConcurrentDevice>>,
  connected_devices: Arc<DashMap<DeviceId, ConcurrentDevice>>,
}

impl DeviceManagerTask {
  pub(crate) fn new(
    cancel_token: CancellationToken,
    transport_managers: Vec<Box<dyn TransportManager>>,
    transport_event_receiver: mpsc::Receiver<TransportManagerEvent>,
    event_sender: broadcast::Sender<DeviceManagerEvent>,
    task_command_receiver: mpsc::Receiver<DeviceManagerCommand>,
    discovered_devices: Arc<DashMap<DeviceId, ConcurrentDevice>>,
    connected_devices: Arc<DashMap<DeviceId, ConcurrentDevice>>,
  ) -> Self {
    Self {
      cancel_token,
      transport_managers,
      transport_event_receiver,
      event_sender,
      task_command_receiver,
      discovered_devices,
      connected_devices,
    }
  }

  #[instrument(skip(self))]
  pub async fn run(&mut self) -> Result<()> {
    loop {
      tokio::select! {
        Some(transport_event) = self.transport_event_receiver.recv() => {
          let _ = self.handle_transport_event(transport_event).await;
        },
        Some(task_command) = self.task_command_receiver.recv() => {
          let _ = self.handle_task_command(task_command).await;
        },
        _ = self.cancel_token.cancelled() => {
          break;
        },
      }
    }

    Ok(())
  }

  #[tracing::instrument(skip(self))]
  async fn handle_transport_event(&mut self, event: TransportManagerEvent) -> Result<()> {
    match event {
      TransportManagerEvent::DeviceDiscovered { ref device } => {
        self.discovered_devices.insert(*device.id(), device.clone());
      }
      TransportManagerEvent::DeviceConnected { ref device } => {
        self.connected_devices.insert(*device.id(), device.clone());
      }
      TransportManagerEvent::DeviceDisconnected(ref device_id) => {
        self.connected_devices.remove(device_id);
        self.discovered_devices.remove(device_id);
      }
      _ => {}
    }

    let manager_event: DeviceManagerEvent = event.try_into()?;
    self.event_sender.send(manager_event).map_err(|err| {
      error!("Error sending event: {}", err);
      err
    })?;

    Ok(())
  }

  #[tracing::instrument(skip(self))]
  async fn handle_task_command(&mut self, command: DeviceManagerCommand) -> Result<()> {
    match command {
      DeviceManagerCommand::ScanStart(sender) => {
        info!("Received scan start command");
        let result = self.start_scanning().await;
        sender.send(result).unwrap_or_else(|_| {
          error!("Error sending scan start response");
        })
      }
      DeviceManagerCommand::ScanStop(sender) => {
        info!("Received scan stop command");
        let result = self.stop_scanning().await;
        sender.send(result).unwrap_or_else(|_| {
          error!("Error sending scan stop response");
        })
      }
    }

    Ok(())
  }

  #[tracing::instrument(skip(self))]
  async fn start_scanning(&mut self) -> Result<()> {
    let futures: Vec<_> = self
      .transport_managers
      .iter_mut()
      .map(|manager| manager.start_scanning())
      .collect();

    join_all(futures)
      .await
      .into_iter()
      .filter_map(|result| result.err())
      .for_each(|err| {
        error!("Error starting scan: {}", err);
      });

    Ok(())
  }

  #[tracing::instrument(skip(self))]
  async fn stop_scanning(&mut self) -> Result<()> {
    let futures: Vec<_> = self
      .transport_managers
      .iter_mut()
      .map(|manager| manager.stop_scanning())
      .collect();

    join_all(futures)
      .await
      .into_iter()
      .filter_map(|result| result.err())
      .for_each(|err| {
        error!("Error stopping scan: {}", err);
      });

    Ok(())
  }
}
