use std::sync::Arc;

use btleplug::platform::Peripheral;

use crate::BHapticsDeviceIdentifier;
use xrc_device_transport_btleplug::api::*;

#[derive(Debug, Clone)]
pub(crate) struct BhapticsDeviceInternal {
  pub(crate) product: Arc<BHapticsDeviceIdentifier>,
  pub(crate) peripheral: Peripheral,
}

#[async_trait::async_trait]
impl DeviceInternal<GenericDeviceProperties> for BhapticsDeviceInternal {
  async fn properties(&self) -> anyhow::Result<Option<GenericDeviceProperties>> {
    return Ok(None);
  }

  async fn connect(&self) -> anyhow::Result<()> {
    todo!()
  }
}
