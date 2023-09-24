use std::fmt::Debug;
use apple_continuity::{
  ContinuityMessage,
  ProximityPairingDeviceModel as DeviceModel,
};
use bytes::Bytes;
use xrc_transport::{
  Result,
  api::Device,
  async_trait,
};
use xrc_transport_btle::btleplug::{BtlePlugProtocolSpecifierBuilder, BtlePlugProtocolSpecifier, PlatformBtlePlugConnector};

struct AppleContinuityDevice {
  id: String,
  device_model: DeviceModel,

  connector: PlatformBtlePlugConnector,
}

impl Debug for AppleContinuityDevice {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AppleContinuityDevice")
      .field("id", &self.id)
      .field("device_model", &self.device_model)
      .finish()
  }
}

impl AppleContinuityDevice {
  fn new(connector: PlatformBtlePlugConnector, device_model: DeviceModel) -> Self {
    Self {
      id: connector.peripheral_info().peripheral_id().to_string(),
      connector,
      device_model,
    }
  }
}

#[async_trait]
impl Device for AppleContinuityDevice {
  fn id(&self) -> &String {
    &self.id
  }

  fn name(&self) -> Option<&String> {
    None // TODO from system connected devices
  }

  async fn update(&mut self) {
    self.connector.update().await;
  }
}

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