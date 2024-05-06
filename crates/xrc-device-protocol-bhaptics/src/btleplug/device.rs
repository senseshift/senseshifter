use anyhow::anyhow;
use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use std::sync::{Arc, RwLock};
use tracing::warn;

use crate::BHapticsDeviceIdentifier;
use xrc_device_transport_btleplug::api::*;

#[derive(Debug, Clone)]
pub(crate) struct BhapticsDeviceInternal {
  pub(crate) product: Arc<BHapticsDeviceIdentifier>,
  pub(crate) peripheral: Peripheral,
  pub(crate) descriptor: Arc<RwLock<GenericDeviceDescriptor>>,
}

#[async_trait::async_trait]
impl DeviceInternal<GenericDeviceProperties> for BhapticsDeviceInternal {
  async fn properties(&self) -> anyhow::Result<Option<GenericDeviceProperties>> {
    return Ok(None);
  }

  async fn connect(&self) -> anyhow::Result<()> {
    if self.peripheral.is_connected().await? {
      return Ok(()); // todo: return error?
    }

    self.peripheral.connect().await?;

    self.peripheral.discover_services().await?;

    if let Ok(sn) = self.read_sn().await {
      // todo: do not panic
      let descriptor = self.descriptor.read().unwrap().clone();
      let descriptor = GenericDeviceDescriptor::new(
        descriptor.name().to_string(),
        descriptor.manufacturer().map(|s| s.to_string()),
        descriptor.product().map(|s| s.to_string()),
        Some(sn),
      );
      *self.descriptor.write().unwrap() = descriptor;
    } else {
      warn!("Could not read SN from device");
    }

    Ok(())
  }
}

impl BhapticsDeviceInternal {
  async fn read_sn(&self) -> crate::Result<String> {
    let sn_char = self
      .peripheral
      .characteristics()
      .into_iter()
      .find(|c| c.uuid == super::constants::CHAR_SN);
    let sn = match sn_char {
      Some(sn_char) => self.peripheral.read(&sn_char).await?,
      None => return Err(anyhow!("Could not find SN characteristic")),
    };

    if sn.len() % 2 != 0 {
      return Err(anyhow!("SN length is not even"));
    }

    // [0xcd, 0x0b, 0x81, 0x45, ...] => "cd0b-8145-..."
    let sn = sn
      .chunks(2)
      .map(|chunk| {
        let chunk = [chunk[0], chunk[1]];
        let chunk = u16::from_be_bytes(chunk.try_into().unwrap());
        format!("{:04x}", chunk)
      })
      .collect::<Vec<_>>()
      .join("-");

    Ok(sn)
  }
}
