use std::fmt::{Debug, Formatter};
use xrc_transport::api::Device;
use xrc_transport::async_trait;
use xrc_transport_btle::btleplug::{BtlePlugDevice, PlatformBtlePlugConnector};
use crate::device_config::BHapticsDeviceType;

pub(crate) struct BHapticsDevice {
  id: String,
  name: String,
  device_model: BHapticsDeviceType,

  connector: PlatformBtlePlugConnector,
}

impl Debug for BHapticsDevice {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("BHapticsDevice")
      .field("id", &self.id)
      .field("device_model", &self.device_model)
      .field("name", &self.name)
      .finish()
  }
}

impl BHapticsDevice {
  pub fn new(
    connector: PlatformBtlePlugConnector,
    name: String,
    device: BHapticsDeviceType
  ) -> Self {
    Self {
      id: connector.peripheral_info().peripheral_id().to_string(),
      name,
      device_model: device,
      connector,
    }
  }
}

#[async_trait]
impl Device for BHapticsDevice {
  fn id(&self) -> &String {
    &self.id
  }

  fn name(&self) -> Option<String> {
    Some(self.name.clone())
  }
}

impl BtlePlugDevice for BHapticsDevice {}