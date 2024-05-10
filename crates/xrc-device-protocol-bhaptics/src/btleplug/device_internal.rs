use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use derivative::Derivative;
use futures::pin_mut;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};
use tracing::{error, info, instrument, Instrument};

use crate::btleplug::device_task::BhapticsDeviceTask;
use crate::BHapticsDeviceIdentifier;
use xrc_device_transport_btleplug::api::*;

#[derive(Derivative, Clone)]
pub(crate) struct BhapticsDeviceInternal {
  name: String,

  product: Arc<BHapticsDeviceIdentifier>,

  #[derivative(Debug = "ignore")]
  peripheral: Peripheral,

  serial_number: Arc<RwLock<Option<String>>>,

  #[derivative(Debug = "ignore")]
  firmware_version: Arc<RwLock<Option<String>>>,

  #[derivative(Debug = "ignore")]
  battery_level: Arc<RwLock<Option<DeviceBatteryProperty>>>,
}

impl Debug for BhapticsDeviceInternal {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("BhapticsDeviceInternal")
      .field("descriptor", &self.descriptor())
      .finish()
  }
}

impl BhapticsDeviceInternal {
  pub fn new(name: String, product: Arc<BHapticsDeviceIdentifier>, peripheral: Peripheral) -> Self {
    Self {
      name,
      product,
      peripheral,

      serial_number: Arc::new(RwLock::new(None)),
      firmware_version: Arc::new(RwLock::new(None)),
      battery_level: Arc::new(RwLock::new(None)),
    }
  }
}

#[async_trait::async_trait]
impl BtlePlugDeviceInternal for BhapticsDeviceInternal {
  fn descriptor(&self) -> GenericDeviceDescriptor {
    return GenericDeviceDescriptor::new(
      self.name.clone(),
      Some("bHaptics".to_string()),
      Some(self.product.device().product_name()),
      self.serial_number.read().unwrap().clone(),
    );
  }

  async fn properties(&self) -> crate::Result<Option<GenericDeviceProperties>> {
    // todo: do not panic
    let result = self.battery_level.read().unwrap().clone();
    let battery_levels = result.as_ref().map(|level| vec![level.clone()]);

    return Ok(Some(GenericDeviceProperties::new(
      None,
      self.firmware_version.read().unwrap().clone(),
      battery_levels,
    )));
  }

  fn connectible(&self) -> bool {
    true
  }

  #[instrument(skip(self))]
  async fn connect(&self) -> crate::Result<()> {
    if self.peripheral.is_connected().await? {
      return Ok(()); // todo: return error?
    }

    self.peripheral.connect().await?;
    self.peripheral.discover_services().await?;

    let sn = self.read_sn().await;
    match self.serial_number.write() {
      Ok(mut serial_number) => *serial_number = sn,
      Err(err) => error!("Could not write serial number: {:?}", err),
    }

    let fw = self.read_firmware_version().await;
    match self.firmware_version.write() {
      Ok(mut firmware_version) => *firmware_version = fw,
      Err(err) => error!("Could not write firmware version: {:?}", err),
    }

    let span = tracing::info_span!("BhapticsDeviceTask", peripheral = %self.peripheral.id());
    let task = BhapticsDeviceTask::new(self.peripheral.clone(), self.battery_level.clone());
    let _task_handle = tokio::spawn(
      async move {
        pin_mut!(task);
        if let Err(err) = task.run().await {
          error!("bHaptics Device Task failed: {:?}", err);
        }
        info!("bHaptics Device Task exited.");
      }
      .instrument(span),
    );

    Ok(())
  }
}

impl BhapticsDeviceInternal {
  async fn read_sn(&self) -> Option<String> {
    let sn_char = self
      .peripheral
      .characteristics()
      .into_iter()
      .find(|c| c.uuid == super::constants::CHAR_SN);

    let sn_char = match sn_char {
      Some(sn_char) => sn_char,
      None => {
        error!("Could not find SN characteristic");
        return None;
      }
    };

    let sn = match self.peripheral.read(&sn_char).await {
      Ok(sn) => sn,
      Err(err) => {
        error!("Could not read SN characteristic: {:?}", err);
        return None;
      }
    };

    if sn.len() % 2 != 0 {
      error!("Invalid SN length: {}", sn.len());
      return None;
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

    Some(sn)
  }

  async fn read_firmware_version(&self) -> Option<String> {
    let fw_char = self
      .peripheral
      .characteristics()
      .into_iter()
      .find(|c| c.uuid == super::constants::CHAR_VERSION);

    let fw_char = match fw_char {
      Some(fw_char) => fw_char,
      None => {
        error!("Could not find FW characteristic");
        return None;
      }
    };

    let fw = match self.peripheral.read(&fw_char).await {
      Ok(fw) => fw,
      Err(err) => {
        error!("Could not read FW characteristic: {:?}", err);
        return None;
      }
    };

    // [0x01, 0x0f] => "1.15"
    let fw = fw
      .iter()
      .map(|&b| format!("{}", b))
      .collect::<Vec<_>>()
      .join(".");

    Some(fw)
  }
}
