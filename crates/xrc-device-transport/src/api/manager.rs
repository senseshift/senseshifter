use crate::Result;
use std::sync::{Arc};
use tokio::sync::{
  RwLock,
  mpsc,
};
use tokio_util::sync::CancellationToken;

use tracing::error;
use crate::api::Device;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransportManagerEvent {
  ScanFinished,
  DeviceDiscovered {
    device_id: String,
    device: Arc<RwLock<Box<dyn Device>>>,
  },
  DeviceUpdated {
    device_id: String,
    device: Arc<RwLock<Box<dyn Device>>>,
  },
}

pub trait TransportManagerBuilder: Send {
  fn finish(&self, event_sender: mpsc::Sender<TransportManagerEvent>) -> Result<Box<dyn TransportManager>>;
}

#[async_trait::async_trait]
pub trait TransportManager: Send + Sync {
  fn name(&self) -> &'static str;

  async fn scan_start(&mut self) -> Result<()>;

  async fn scan_stop(&mut self) -> Result<()>;

  fn is_scanning(&self) -> bool {
    false
  }

  fn ready(&self) -> bool {
    true
  }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait RescanTransportManager: Sync + Send {
  fn name(&self) -> &'static str;

  fn ready(&self) -> bool;

  fn rescan_wait_duration(&self) -> std::time::Duration;

  async fn scan(&self) -> Result<()>;

}

pub(crate) struct PeriodicScanTransportManager<T: RescanTransportManager + 'static> {
  inner: Arc<T>,
  cancel_token: Option<CancellationToken>,
}

impl<T: RescanTransportManager> PeriodicScanTransportManager<T> {
  pub fn new(inner: T) -> Self {
    Self {
      inner: Arc::new(inner),
      cancel_token: None,
    }
  }
}

#[async_trait::async_trait]
impl<T: RescanTransportManager> TransportManager for PeriodicScanTransportManager<T> {
  fn name(&self) -> &'static str {
    self.inner.name()
  }

  async fn scan_start(&mut self) -> Result<()> {
    if self.cancel_token.is_some() {
      return Ok(());
    }

    let cancel_token = CancellationToken::new();
    let child_token = cancel_token.child_token();
    self.cancel_token = Some(cancel_token);

    let inner = self.inner.clone();
    tokio::spawn(async move {
      loop {
        if let Err(err) = inner.scan().await {
          error!("PeriodicScanTransportManager Failure: {}", err);
          break;
        }

        // Wait for the next scan or cancellation.
        tokio::select! {
          _ = tokio::time::sleep(inner.rescan_wait_duration()) => continue,
          _ = child_token.cancelled() => break,
        }
      }
    });

    Ok(())
  }

  async fn scan_stop(&mut self) -> Result<()> {
    if self.cancel_token.is_none() {
      return Ok(());
    }
    Ok(self.cancel_token.take().unwrap().cancel())
  }

  fn is_scanning(&self) -> bool {
    self.cancel_token.is_some()
  }

  fn ready(&self) -> bool {
    self.inner.ready()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_periodic_scan() {
    let mut mock = MockRescanTransportManager::new();

    mock.expect_ready()
      .times(1)
      .return_once(|| false);

    mock.expect_ready()
      .times(1)
      .return_once(|| true);

    mock.expect_name().returning(|| "test");
    mock.expect_rescan_wait_duration().returning(|| std::time::Duration::from_millis(1));

    let mut manager = PeriodicScanTransportManager::new(mock);

    assert!(!manager.ready());
    assert!(manager.ready());

    assert_eq!("test", manager.name());
    assert_eq!(std::time::Duration::from_millis(1), manager.inner.rescan_wait_duration());

    assert!(!manager.is_scanning());
  }
}