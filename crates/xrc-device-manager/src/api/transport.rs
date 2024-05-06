use crate::api::*;
use crate::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

pub trait TransportManagerBuilder: Default + Send {
  fn finish(
    &self,
    event_sender: mpsc::Sender<TransportManagerEvent>,
  ) -> Result<Box<dyn TransportManager>>;
}

#[async_trait::async_trait]
pub trait TransportManager: Send + Sync {
  fn name(&self) -> &'static str;

  async fn start_scanning(&self) -> Result<()>;

  async fn stop_scanning(&self) -> Result<()>;

  async fn connect(&self, device_id: &DeviceId) -> Result<()>;
}

#[derive(Debug)]
pub enum TransportManagerEvent {
  /// Scan started
  ScanStarted,
  /// Continuous scan stopped
  ScanStopped,

  DeviceDiscovered {
    device: Arc<GenericDevice<GenericDeviceDescriptor, GenericDeviceProperties>>,
  },
  DeviceUpdated {
    device: Arc<GenericDevice<GenericDeviceDescriptor, GenericDeviceProperties>>,
  },

  DeviceConnected {
    device: Arc<GenericDevice<GenericDeviceDescriptor, GenericDeviceProperties>>,
  },
  DeviceDisconnected(DeviceId),
}
