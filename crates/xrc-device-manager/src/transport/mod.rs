use std::fmt::Debug;

use crate::Result;

use tokio::sync::mpsc;
use dyn_clone::DynClone;

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

pub trait DeviceCandidate: Send + Sync + Debug + DynClone {
  fn id(&self) -> String;

  fn display_name(&self) -> String {
    self.id()
  }

  fn connectible(&self) -> bool {
    false
  }
}

pub trait Device: DeviceCandidate {}

#[derive(Debug)]
pub enum TransportManagerEvent {
  /// Scan started
  ScanStarted,
  /// Continuous scan stopped
  ScanStopped,
  /// Scan successfully finished (for periodic scans)
  ScanFinished,

  DeviceDiscovered(String),
  DeviceUpdated(String),
}
