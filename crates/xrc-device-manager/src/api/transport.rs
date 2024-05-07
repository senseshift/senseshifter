use crate::api::*;
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

#[cfg(any(feature = "mockall", test))]
use mockall::{automock, predicate::*};

pub trait TransportManagerBuilder {
  fn finish(
    &self,
    event_sender: mpsc::Sender<TransportManagerEvent>,
  ) -> Result<Box<dyn TransportManager>>;
}

#[async_trait]
pub trait TransportManager: Send + Sync {
  fn name(&self) -> &'static str;

  fn ready(&self) -> bool {
    true
  }

  fn is_scanning(&self) -> bool {
    false
  }

  async fn start_scanning(&mut self) -> Result<()>;

  async fn stop_scanning(&mut self) -> Result<()>;

  async fn connect_scanned(&self, device_id: &DeviceId) -> Result<()>;

  fn devices(&self) -> Result<Vec<ConcurrentDevice>>;

  fn get_device(&self, device_id: &DeviceId) -> Result<Option<ConcurrentDevice>>;
}

#[cfg_attr(any(feature = "mockall", test), automock)]
#[async_trait]
pub trait RescanTransportManager: Sync + Send {
  fn name(&self) -> &'static str;

  fn rescan_wait_duration(&self) -> std::time::Duration;

  fn ready(&self) -> bool;

  async fn scan(&self) -> Result<()>;

  async fn connect_scanned(&self, device_id: &DeviceId) -> Result<()>;

  fn devices(&self) -> Result<Vec<ConcurrentDevice>>;

  fn get_device(&self, device_id: &DeviceId) -> Result<Option<ConcurrentDevice>>;
}

pub struct PeriodicScanTransportManager<T: RescanTransportManager + 'static> {
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

#[async_trait]
impl<T: RescanTransportManager> TransportManager for PeriodicScanTransportManager<T> {
  #[inline(always)]
  fn name(&self) -> &'static str {
    self.inner.name()
  }

  fn ready(&self) -> bool {
    self.inner.ready()
  }

  fn is_scanning(&self) -> bool {
    self.cancel_token.is_some()
  }

  async fn start_scanning(&mut self) -> Result<()> {
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

  async fn stop_scanning(&mut self) -> Result<()> {
    if self.cancel_token.is_none() {
      return Ok(());
    }
    self.cancel_token.take().unwrap().cancel();
    Ok(())
  }

  #[inline(always)]
  async fn connect_scanned(&self, device_id: &DeviceId) -> Result<()> {
    self.inner.connect_scanned(device_id).await
  }

  #[inline(always)]
  fn devices(&self) -> Result<Vec<ConcurrentDevice>> {
    self.inner.devices()
  }

  #[inline(always)]
  fn get_device(&self, device_id: &DeviceId) -> Result<Option<ConcurrentDevice>> {
    self.inner.get_device(device_id)
  }
}

#[derive(Debug)]
pub enum TransportManagerEvent {
  DeviceDiscovered { device: ConcurrentDevice },
  DeviceUpdated { device: ConcurrentDevice },

  DeviceConnected { device: ConcurrentDevice },
  DeviceDisconnected(DeviceId),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_periodic_scan() {
    let mut mock = MockRescanTransportManager::new();

    mock.expect_ready().times(1).return_once(|| false);

    mock.expect_ready().times(1).return_once(|| true);

    mock.expect_name().returning(|| "test");
    mock
      .expect_rescan_wait_duration()
      .returning(|| std::time::Duration::from_millis(1));

    let manager = PeriodicScanTransportManager::new(mock);

    assert!(!manager.ready());
    assert!(manager.ready());

    assert_eq!("test", manager.name());
    assert_eq!(
      std::time::Duration::from_millis(1),
      manager.inner.rescan_wait_duration()
    );

    assert!(!manager.is_scanning());
  }
}
