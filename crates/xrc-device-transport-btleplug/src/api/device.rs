use crate::api::*;
use crate::Result;
use async_trait::async_trait;
use btleplug::api::Peripheral;
use derivative::Derivative;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct BtlePlugDevice<P: Peripheral> {
  id: DeviceId,

  #[derivative(Debug = "ignore")]
  peripheral: P,

  connected: AtomicBool,

  internal: Box<dyn BtlePlugDeviceInternal>,
}

impl<P: Peripheral> BtlePlugDevice<P> {
  pub fn new(id: DeviceId, peripheral: P, internal: Box<dyn BtlePlugDeviceInternal>) -> Self {
    Self {
      id,
      peripheral,
      connected: AtomicBool::new(false),
      internal,
    }
  }

  pub async fn handle_update_event(&self) -> Result<()> {
    self.internal.handle_updated().await
  }
}

#[async_trait]
impl<P: Peripheral> Device<GenericDeviceDescriptor, GenericDeviceProperties> for BtlePlugDevice<P> {
  fn id(&self) -> &DeviceId {
    &self.id
  }

  fn descriptor(&self) -> GenericDeviceDescriptor {
    self.internal.descriptor()
  }

  async fn properties(&self) -> Result<Option<GenericDeviceProperties>> {
    self.internal.properties().await
  }

  fn connectible(&self) -> bool {
    self.internal.connectible()
  }

  async fn is_connected(&self) -> bool {
    self.peripheral.is_connected().await.unwrap_or(false)
  }

  async fn connect(&self) -> Result<()> {
    self.internal.connect().await?;

    self.connected.store(true, Ordering::Relaxed);

    Ok(())
  }
}

#[cfg_attr(any(feature = "mockall", test), mockall::automock)]
#[async_trait]
pub trait BtlePlugDeviceInternal: Send + Sync + Debug {
  fn descriptor(&self) -> GenericDeviceDescriptor;

  async fn properties(&self) -> Result<Option<GenericDeviceProperties>>;

  fn connectible(&self) -> bool {
    false
  }

  async fn handle_updated(&self) -> Result<()> {
    Ok(())
  }

  async fn connect(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
  use super::*;
  use mockall::mock;

  mock! {
    Peripheral {}
  }

  #[test]
  fn test_wrapper() {
    let mut mock = MockBtlePlugDeviceInternal::new();

    mock.expect_descriptor().returning(|| {
      GenericDeviceDescriptor::new(
        "test".to_string(),
        Some("test manufacturer".to_string()),
        Some("test product".to_string()),
        None,
      )
    });
    mock.expect_properties().returning(|| Ok(None));
    mock.expect_connectible().returning(|| false);
    mock.expect_handle_updated().returning(|| Ok(()));
    mock.expect_connect().returning(|| Ok(()));
  }
}
