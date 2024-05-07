use anyhow::anyhow;
use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use std::sync::{Arc, RwLock};
use tracing::{instrument, warn};

use crate::btleplug::device_task::BhapticsDeviceTask;
use crate::BHapticsDeviceIdentifier;
use xrc_device_transport_btleplug::api::*;

#[derive(Debug, Clone)]
pub(crate) struct BhapticsDeviceInternal {
  product: Arc<BHapticsDeviceIdentifier>,
  peripheral: Peripheral,
  descriptor: Arc<RwLock<GenericDeviceDescriptor>>,

  firmware_version: Arc<RwLock<Option<String>>>,
  battery_level: Arc<RwLock<Option<DeviceBatteryProperty>>>,
}

impl BhapticsDeviceInternal {
  pub fn new(
    product: Arc<BHapticsDeviceIdentifier>,
    peripheral: Peripheral,
    descriptor: Arc<RwLock<GenericDeviceDescriptor>>,
  ) -> Self {
    Self {
      product,
      peripheral,
      descriptor,

      firmware_version: Arc::new(RwLock::new(None)),
      battery_level: Arc::new(RwLock::new(None)),
    }
  }
}

#[async_trait::async_trait]
impl DeviceInternal<GenericDeviceProperties> for BhapticsDeviceInternal {
  async fn properties(&self) -> crate::Result<Option<GenericDeviceProperties>> {
    // todo: do not panic
    let result = self.battery_level.read().unwrap().clone();
    let battery_levels = result.as_ref().map(|level| vec![level.clone()]);

    return Ok(Some(GenericDeviceProperties::new(
      self.firmware_version.read().unwrap().clone(),
      None,
      battery_levels,
    )));
  }

  #[instrument(skip(self))]
  async fn connect(&self) -> crate::Result<()> {
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

    // todo: do not panic
    *self.firmware_version.write().unwrap() = self.read_firmware_version().await.ok();

    let task = BhapticsDeviceTask::new(self.peripheral.clone(), self.battery_level.clone());

    tokio::spawn(async move {
      task.run().await.ok();
    });

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
        let chunk = u16::from_be_bytes(chunk);
        format!("{:04x}", chunk)
      })
      .collect::<Vec<_>>()
      .join("-");

    Ok(sn)
  }

  async fn read_firmware_version(&self) -> crate::Result<String> {
    let fw_char = self
      .peripheral
      .characteristics()
      .into_iter()
      .find(|c| c.uuid == super::constants::CHAR_VERSION);
    let fw = match fw_char {
      Some(fw_char) => self.peripheral.read(&fw_char).await?,
      None => return Err(anyhow!("Could not find FW characteristic")),
    };

    if fw.len() % 2 != 0 {
      return Err(anyhow!("FW length is not even"));
    }

    // [0x01, 0x0f] => "1.15"
    let fw = fw
      .iter()
      .map(|&b| format!("{}", b))
      .collect::<Vec<_>>()
      .join(".");

    Ok(fw)
  }
}
