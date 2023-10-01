use std::sync::Arc;
use tracing::info;
use xrc_transport::{
  Result,
  api::Device,
  async_trait,
};
use xrc_transport_btle::btleplug::{BtlePlugProtocolSpecifierBuilder, BtlePlugProtocolSpecifier, PlatformBtlePlugConnector, BtlePlugDevice};

mod device;
use device::*;
mod constants;
mod device_config;
use device_config::BHapticsDeviceIdentifier;

#[derive(Default)]
pub struct BHapticsProtocolSpecifierBuilder {}

impl BtlePlugProtocolSpecifierBuilder for BHapticsProtocolSpecifierBuilder {
  fn finish(&self) -> Box<dyn BtlePlugProtocolSpecifier> {
    let identifiers = device_config::load_device_identifiers()
      .iter_mut()
      .map(|identifier| Arc::new(identifier.clone()))
      .collect();

    Box::new(BHapticsProtocolSpecifier {
      device_identifiers: identifiers,
    })
  }
}


pub struct BHapticsProtocolSpecifier {
  device_identifiers: Vec<Arc<BHapticsDeviceIdentifier>>,
}

impl BHapticsProtocolSpecifier {
  fn get_product(&self, name: &String, appearance: &Option<u16>) -> Option<Arc<BHapticsDeviceIdentifier>> {
    if appearance.is_some() && (appearance.unwrap() == 509 || appearance.unwrap() == 510) {
      return self.get_product_by_appearance(appearance);
    }
    self.get_product_by_name(name)
  }

  fn get_product_by_appearance(&self, appearance: &Option<u16>) -> Option<Arc<BHapticsDeviceIdentifier>> {
    let appearance = match appearance {
      Some(appearance) => appearance,
      None => {
        return None;
      }
    };

    self
      .device_identifiers
      .iter()
      .find(|identifier| identifier.appearance() == appearance)
      .map_or(None, |identifier| Some(identifier.clone()))
  }

  fn get_product_by_name(&self, name: &String) -> Option<Arc<BHapticsDeviceIdentifier>> {
    let lower_name = name.clone().to_lowercase();

    self
      .device_identifiers
      .iter()
      .find(|identifier| lower_name.contains(identifier.name_contains()))
      .map_or(None, |identifier| Some(identifier.clone()))
  }
}

#[async_trait]
impl BtlePlugProtocolSpecifier for BHapticsProtocolSpecifier {
  #[tracing::instrument(skip(self, connector))]
  fn specify(&self, connector: PlatformBtlePlugConnector) -> Result<Option<Box<dyn BtlePlugDevice>>> {
    let info = connector.peripheral_info();
    let properties = match info.properties() {
      Some(properties) => properties,
      None => return Ok(None),
    };

    let name = match &properties.local_name {
      Some(name) => name,
      None => return Ok(None),
    };

    let product = match self.get_product(name, &properties.appearance) {
      Some(product) => product,
      None => return Ok(None),
    };

    let name = name.clone();
    let device = BHapticsDevice::new(
      connector,
      name,
      product.device().clone(),
    );

    Ok(Some(Box::new(device)))
  }
}