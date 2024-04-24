use async_trait::async_trait;
use btleplug::api::{Peripheral as _, PeripheralProperties};
use btleplug::platform::Peripheral;
use tracing::{info, instrument};
use crate::Result;
use crate::transport::DeviceCandidate;
use super::{BtlePlugDeviceCandidate, BtlePlugProtocolHandler, BtlePlugProtocolHandlerBuilder};

#[derive(Debug, Clone)]
pub(super) struct BhapticsDevice {
  peripheral: Peripheral,
  name: String,
}

impl DeviceCandidate for BhapticsDevice {
  fn id(&self) -> String {
    self.peripheral.address().to_string()
  }

  fn display_name(&self) -> String {
    // Here we assume that device was specified correctly and local_name is not None
    self.name.clone()
  }
}

#[async_trait::async_trait]
impl BtlePlugDeviceCandidate for BhapticsDevice {
  #[instrument(skip(self))]
  async fn update_properties(&mut self) -> Result<()> {
    info!("Updating properties for bhaptics device");
    Ok(())
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
  fn specify_protocol(&self, peripheral: Peripheral, properties: Option<PeripheralProperties>) -> Result<Option<Box<dyn BtlePlugDeviceCandidate>>> {
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
