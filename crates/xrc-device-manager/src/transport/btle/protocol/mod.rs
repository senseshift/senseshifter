use async_trait::async_trait;
use btleplug::api::PeripheralProperties;
use btleplug::platform::Peripheral;
use crate::Result;
use crate::transport::DeviceCandidate;

pub mod bhaptics;

#[async_trait::async_trait]
pub trait BtlePlugDeviceCandidate: DeviceCandidate {
  async fn update_properties(&mut self) -> Result<()> {
    Ok(())
  }
}

pub trait BtlePlugProtocolHandlerBuilder: Send {
  fn finish(&self) -> Box<dyn BtlePlugProtocolHandler>;
}

#[async_trait::async_trait]
pub trait BtlePlugProtocolHandler: Send + Sync {
  fn name(&self) -> &'static str;

  fn specify_protocol(&self, peripheral: Peripheral, peripheral_properties: Option<PeripheralProperties>) -> Result<Option<Box<dyn BtlePlugDeviceCandidate>>>;
}
