use async_trait::async_trait;
use btleplug::api::{Peripheral as _, PeripheralProperties};
use btleplug::platform::Peripheral;
use tracing::{info, instrument};
use crate::Result;
use crate::transport::btle::api::*;

#[derive(Debug, Clone)]
pub(super) struct BhapticsDevice {
  peripheral: Peripheral,
  name: String,
}

impl Device for BhapticsDevice {
  fn id(&self) -> String {
    self.peripheral.id().to_string()
  }

  fn name(&self) -> String {
    self.name.clone()
  }

  fn connectible(&self) -> bool {
    true
  }
}

#[derive(Default)]
pub struct BhapticsProtocolHandlerBuilder {}

impl BtlePlugProtocolHandlerBuilder for BhapticsProtocolHandlerBuilder {
  fn finish(&self) -> Box<dyn BtlePlugProtocolHandler> {
    Box::new(BhapticsProtocolHandler {})
  }
}

pub struct BhapticsProtocolHandler {}

#[async_trait::async_trait]
impl BtlePlugProtocolHandler for BhapticsProtocolHandler {
  fn name(&self) -> &'static str {
    "bhaptics"
  }

  #[instrument(skip(self, peripheral))]
  fn specify_protocol(&self, peripheral: Peripheral, properties: Option<PeripheralProperties>) -> Result<Option<Box<dyn Device>>> {
    let properties = match properties {
      Some(properties) => properties,
      None => return Ok(None),
    };

    let name = match properties.local_name {
      Some(ref name) => name.clone(),
      None => return Ok(None),
    };

    let appearance = match properties.appearance {
      Some(appearance) => appearance,
      None => return Ok(None),
    };

    if appearance == 508 {
      return Ok(Some(Box::new(BhapticsDevice {
        peripheral,
        name,
      })));
    }

    Ok(None)
  }
}
