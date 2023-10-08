use anyhow::Result;

use xrc_transport::{
  async_trait,
  api::Device,
};

use crate::btleplug::{PlatformBtlePlugConnector};

pub trait BtlePlugProtocolSpecifierBuilder: Send {
  fn finish(&self) -> Box<dyn BtlePlugProtocolSpecifier>;
}

#[async_trait]
pub trait BtlePlugProtocolSpecifier: Send + Sync {
  fn specify(&self, connector: PlatformBtlePlugConnector) -> Result<Option<Box<dyn Device>>>;
}