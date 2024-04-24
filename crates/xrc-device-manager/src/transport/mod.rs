use std::fmt::Debug;

use crate::Result;

use tokio::sync::mpsc;
use dyn_clone::DynClone;
use crate::transport::btle::api::Device;

pub mod btle;

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
}

#[derive(Debug)]
pub enum TransportManagerEvent {
  /// Scan started
  ScanStarted,
  /// Continuous scan stopped
  ScanStopped,

  DeviceDiscovered {
    id: String,
    device: Box<dyn Device>,
  },
  DeviceUpdated {
    id: String,
    device: Box<dyn Device>,
  },
}
