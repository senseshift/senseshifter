use crate::Result;

use btleplug::platform::Peripheral;

use super::*;

pub trait BtlePlugProtocolHandlerBuilder: Send {
  fn finish(&self) -> Box<dyn BtlePlugProtocolHandler>;
}

#[async_trait::async_trait]
pub trait BtlePlugProtocolHandler: Send + Sync {
  fn name(&self) -> &'static str;

  async fn specify_protocol(&self, peripheral: Peripheral) -> Result<Option<Box<dyn Device>>>;

  async fn connect_peripheral(&self, peripheral: Peripheral) -> Result<()>;
}
