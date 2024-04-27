use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use tracing::instrument;
use xrc_device_transport_btleplug::api::*;

use crate::BhapticsDeviceConnector;

#[derive(Debug, Clone)]
pub(crate) struct BhapticsDevice {
  pub(crate) peripheral: Peripheral,
  pub(crate) name: String,

  pub(crate) connector: BhapticsDeviceConnector,

  pub(crate) connected: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl Device for BhapticsDevice {
  fn id(&self) -> DeviceId {
    self.peripheral.id().to_string()
  }

  fn name(&self) -> String {
    self.name.clone()
  }

  fn connectible(&self) -> bool {
    true
  }

  fn connected(&self) -> bool {
    self.connected.load(Ordering::SeqCst)
  }

  #[instrument(skip(self))]
  async fn connect(&self) -> crate::Result<()> {
    self.connector.connect().await?;
    self.connected.store(true, Ordering::SeqCst);
    Ok(())
  }
}
