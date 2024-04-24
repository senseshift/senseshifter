use btleplug::api::PeripheralProperties;
use btleplug::platform::Peripheral;
use crate::Result;
use crate::transport::DeviceCandidate;

pub mod bhaptics;

pub trait BtlePlugDeviceCandidate: DeviceCandidate {}

pub trait BtlePlugProtocolHandlerBuilder: Send {
  fn finish(&self) -> Box<dyn BtlePlugProtocolHandler>;
}

#[async_trait::async_trait]
pub trait BtlePlugProtocolHandler: Send + Sync {
  fn name(&self) -> &'static str;

  fn specify_protocol(&self, peripheral: Peripheral, peripheral_properties: Option<PeripheralProperties>) -> Result<Option<Box<dyn BtlePlugDeviceCandidate>>>;
}
