use std::sync::Arc;
use btleplug::api::Peripheral;
use crate::transport::Device;

#[derive(Debug, Clone)]
pub(super) struct BtlePlugPeripheral {
  pub(super) peripheral: btleplug::platform::Peripheral,
}

impl Device for BtlePlugPeripheral {
  fn id(&self) -> String {
    self.peripheral.id().to_string()
  }
}