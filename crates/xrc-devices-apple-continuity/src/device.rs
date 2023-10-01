use std::fmt::Debug;
use apple_continuity::{
  ProximityPairingDeviceModel as DeviceModel,
};
use xrc_transport::api::Device;
use xrc_transport::async_trait;
use xrc_transport_btle::btleplug::{BtlePlugDevice, PlatformBtlePlugConnector};

pub(crate) struct AppleContinuityDevice {
  id: String,
  device_model: DeviceModel,
}

impl Debug for AppleContinuityDevice {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AppleContinuityDevice")
      .field("id", &self.id)
      .field("device_model", &self.device_model)
      .finish()
  }
}

impl AppleContinuityDevice {
  pub fn new(connector: PlatformBtlePlugConnector, device_model: DeviceModel) -> Self {
    Self {
      id: connector.peripheral_info().peripheral_id().to_string(),
      device_model,
    }
  }
}

#[async_trait]
impl Device for AppleContinuityDevice {
  fn id(&self) -> &String {
    &self.id
  }

  fn name(&self) -> Option<String> {
    None // TODO from system connected devices
  }
}

impl BtlePlugDevice for AppleContinuityDevice {}