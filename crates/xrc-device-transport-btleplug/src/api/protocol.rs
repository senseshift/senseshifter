use crate::Result;

use btleplug::platform::Peripheral;

use super::*;

pub trait BtlePlugProtocolSpecifierBuilder: Send {
  fn finish(&self) -> Box<dyn BtlePlugProtocolSpecifier>;
}

#[async_trait::async_trait]
pub trait BtlePlugProtocolSpecifier: Send + Sync {
  fn name(&self) -> &'static str;

  async fn specify_protocol(
    &self,
    peripheral: Peripheral,
  ) -> Result<Option<Box<dyn BtlePlugDeviceInternal>>>;
}
