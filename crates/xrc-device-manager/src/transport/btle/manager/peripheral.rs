use crate::transport::DeviceCandidate;
use btleplug::api::Peripheral;

#[derive(Debug, Clone)]
pub(super) struct BtlePlugPeripheral {
  pub(super) peripheral: btleplug::platform::Peripheral,
}

impl DeviceCandidate for BtlePlugPeripheral {
  fn id(&self) -> String {
    self.peripheral.id().to_string()
  }
}
