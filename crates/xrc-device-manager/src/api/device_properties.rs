use derivative::Derivative;

#[cfg(any(feature = "mockall", test))]
use mockall::{automock, predicate::*};

#[cfg_attr(any(feature = "mockall", test), automock)]
pub trait DeviceProperties {
  fn hardware_version(&self) -> &String;

  fn firmware_version(&self) -> &String;

  // todo: add battery_levels
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative, Debug, Clone)]
pub struct GenericDeviceProperties {
  hardware_version: String,
  firmware_version: String,
}

impl GenericDeviceProperties {
  #[inline(always)]
  pub fn new(hardware_version: String, firmware_version: String) -> Self {
    Self {
      hardware_version,
      firmware_version,
    }
  }
}

impl DeviceProperties for GenericDeviceProperties {
  #[inline(always)]
  fn hardware_version(&self) -> &String {
    &self.hardware_version
  }

  #[inline(always)]
  fn firmware_version(&self) -> &String {
    &self.firmware_version
  }
}
