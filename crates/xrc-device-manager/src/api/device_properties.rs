use derivative::Derivative;
use getset::{Getters, Setters};
use std::fmt::Debug;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derivative(Default)]
pub enum DeviceBatteryStatus {
  Charging,
  Discharging,
  Full,
  NotCharging,
  #[derivative(Default)]
  Unknown,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative, Setters, Getters, Debug, Clone)]
#[derivative(Default)]
pub struct DeviceBatteryProperty {
  #[getset(get = "pub", set = "pub")]
  level: f32,
  #[getset(get = "pub", set = "pub")]
  status: DeviceBatteryStatus,
  #[getset(get = "pub", set = "pub")]
  voltage: Option<f32>,
}

impl DeviceBatteryProperty {
  #[inline(always)]
  pub fn new(level: f32, status: DeviceBatteryStatus, voltage: Option<f32>) -> Self {
    Self {
      level,
      status,
      voltage,
    }
  }
}

pub trait DeviceProperties: Debug {
  fn hardware_version(&self) -> Option<&str>;

  fn firmware_version(&self) -> Option<&str>;

  fn battery_levels(&self) -> Option<&Vec<DeviceBatteryProperty>> {
    None
  }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative, Debug, Clone)]
#[derivative(Default)]
pub struct GenericDeviceProperties {
  hardware_version: Option<String>,
  firmware_version: Option<String>,
  battery_levels: Option<Vec<DeviceBatteryProperty>>,
}

impl GenericDeviceProperties {
  #[inline(always)]
  pub fn new(
    hardware_version: Option<String>,
    firmware_version: Option<String>,
    battery_levels: Option<Vec<DeviceBatteryProperty>>,
  ) -> Self {
    Self {
      hardware_version,
      firmware_version,
      battery_levels,
    }
  }
}

impl DeviceProperties for GenericDeviceProperties {
  #[inline(always)]
  fn hardware_version(&self) -> Option<&str> {
    self.hardware_version.as_deref()
  }

  #[inline(always)]
  fn firmware_version(&self) -> Option<&str> {
    self.firmware_version.as_deref()
  }

  #[inline(always)]
  fn battery_levels(&self) -> Option<&Vec<DeviceBatteryProperty>> {
    self.battery_levels.as_ref()
  }
}
