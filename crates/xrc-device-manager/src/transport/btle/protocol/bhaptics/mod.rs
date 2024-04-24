use btleplug::api::PeripheralProperties;
use btleplug::platform::Peripheral;
use tracing::{info, instrument};
use crate::Result;
use super::{BtlePlugDeviceCandidate, BtlePlugProtocolHandler, BtlePlugProtocolHandlerBuilder};

#[derive(Default)]
pub struct BhapticsProtocolHandlerBuilder {}

impl BtlePlugProtocolHandlerBuilder for BhapticsProtocolHandlerBuilder {
  fn finish(&self) -> Box<dyn BtlePlugProtocolHandler> {
    Box::new(BhapticsProtocolHandler {})
  }
}

pub struct BhapticsProtocolHandler {}

impl BtlePlugProtocolHandler for BhapticsProtocolHandler {
  fn name(&self) -> &'static str {
    "bhaptics"
  }

  #[instrument(skip(self, peripheral))]
  fn specify_protocol(&self, peripheral: Peripheral, properties: Option<PeripheralProperties>) -> Result<Option<Box<dyn BtlePlugDeviceCandidate>>> {
    let properties = match properties {
      Some(properties) => properties,
      None => return Ok(None),
    };

    let name = match properties.local_name {
      Some(name) => name,
      None => return Ok(None),
    };

    let appearance = match properties.appearance {
      Some(appearance) => appearance,
      None => return Ok(None),
    };

    if appearance == 508 {
      info!("Found bhaptics device: {}", name);
    }

    Ok(None)
  }
}
