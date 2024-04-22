use std::fmt::Debug;

use crate::Result;

use tokio::sync::mpsc;

pub mod btle;

pub trait TransportManagerBuilder: Send {
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

pub trait Device: Send + Sync + Debug {
  fn id(&self) -> String;
}

#[derive(Debug)]
pub enum TransportManagerEvent {
  /// Scan started
  ScanStarted,
  /// Continuous scan stopped
  ScanStopped,
  /// Scan successfully finished (for periodic scans)
  ScanFinished,

  DeviceDiscovered(Box<dyn Device>),
  DeviceUpdated(Box<dyn Device>),
}
