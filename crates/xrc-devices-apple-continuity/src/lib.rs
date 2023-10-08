use std::fmt::Debug;
use btleplug::api::{BDAddr, Peripheral};
use apple_continuity::{
  ContinuityMessage,
};
use bytes::Bytes;
use tracing::{error, info};
use xrc_transport::{
  Result,
  api::Device,
  async_trait,
};
use xrc_transport_btle::btleplug::{BtlePlugProtocolSpecifierBuilder, BtlePlugProtocolSpecifier, PlatformBtlePlugConnector};

mod device;
use device::*;

#[derive(Default)]
pub struct AppleContinuityProtocolSpecifierBuilder {}

impl BtlePlugProtocolSpecifierBuilder for AppleContinuityProtocolSpecifierBuilder {
  fn finish(&self) -> Box<dyn BtlePlugProtocolSpecifier> {
    Box::new(AppleContinuityProtocolSpecifier {})
  }
}

pub struct AppleContinuityProtocolSpecifier {
  //
}

#[async_trait]
impl BtlePlugProtocolSpecifier for AppleContinuityProtocolSpecifier {
  fn specify(&self, connector: PlatformBtlePlugConnector) -> Result<Option<Box<dyn Device>>> {
    let info = connector.peripheral_info();
    let properties = match info.properties() {
      Some(properties) => properties,
      None => return Ok(None),
    };

    let message = match properties.manufacturer_data.get(&76) {
      Some(message) => message,
      None => return Ok(None),
    };
    let message = match ContinuityMessage::try_from(Bytes::from(message.clone())) {
      Ok(message) => message,
      Err(_) => {
        return Ok(None);
      }
    };
    let message = match message {
      ContinuityMessage::ProximityPairing(message) => message,
      _ => return Ok(None),
    };

    let device = AppleContinuityDevice::new(
      connector,
      message.device_model,
    );

    Ok(Some(Box::new(device)))
  }
}