use crate::device_config::{load_device_identifiers, BHapticsDeviceIdentifier};
use crate::BhapticsDevice;
use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use std::sync::Arc;
use tracing::instrument;
use xrc_device_transport_btleplug::api::{
  BtlePlugProtocolSpecifier, BtlePlugProtocolSpecifierBuilder, Device,
};

#[derive(Default)]
pub struct BhapticsProtocolSpecifierBuilder {}

impl BtlePlugProtocolSpecifierBuilder for BhapticsProtocolSpecifierBuilder {
  fn finish(&self) -> Box<dyn BtlePlugProtocolSpecifier> {
    let device_identifiers = load_device_identifiers()
      .iter_mut()
      .map(|identifier| Arc::new(identifier.clone()))
      .collect();

    Box::new(BhapticsProtocolSpecifier { device_identifiers })
  }
}

pub struct BhapticsProtocolSpecifier {
  device_identifiers: Vec<Arc<BHapticsDeviceIdentifier>>,
}

#[async_trait::async_trait]
impl BtlePlugProtocolSpecifier for BhapticsProtocolSpecifier {
  fn name(&self) -> &'static str {
    "bhaptics"
  }

  #[instrument(skip(self, peripheral))]
  async fn specify_protocol(
    &self,
    peripheral: Peripheral,
  ) -> crate::Result<Option<Box<dyn Device>>> {
    let properties = match peripheral.properties().await? {
      Some(properties) => properties,
      None => return Ok(None),
    };

    let name = match properties.local_name {
      Some(ref name) => name.clone(),
      None => return Ok(None),
    };

    let product = self.get_product(&name, &properties.appearance);

    if product.is_some() {
      return Ok(Some(Box::new(BhapticsDevice { peripheral, name })));
    }

    Ok(None)
  }

  #[instrument(skip(self))]
  async fn connect_peripheral(&self, _peripheral: Peripheral) -> crate::Result<()> {
    Ok(())
  }
}

impl BhapticsProtocolSpecifier {
  fn get_product(
    &self,
    name: &String,
    appearance: &Option<u16>,
  ) -> Option<Arc<BHapticsDeviceIdentifier>> {
    if appearance.is_some() && (appearance.unwrap() == 509 || appearance.unwrap() == 510) {
      return self.get_product_by_appearance(appearance);
    }
    self.get_product_by_name(name)
  }

  fn get_product_by_appearance(
    &self,
    appearance: &Option<u16>,
  ) -> Option<Arc<BHapticsDeviceIdentifier>> {
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
      .cloned()
  }

  fn get_product_by_name(&self, name: &String) -> Option<Arc<BHapticsDeviceIdentifier>> {
    let lower_name = name.clone().to_lowercase();

    self
      .device_identifiers
      .iter()
      .find(|identifier| lower_name.contains(identifier.name_contains()))
      .cloned()
  }
}
