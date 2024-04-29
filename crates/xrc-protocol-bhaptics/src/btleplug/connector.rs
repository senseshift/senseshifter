use super::constants::{CHAR_BATTERY, CHAR_SN, CHAR_VERSION};
use super::device_task::BhapticsDeviceTask;
use crate::Result;
use anyhow::{anyhow, Context};
use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use tracing::{error, info, instrument, warn};

#[derive(Debug, Clone)]
pub(crate) struct BhapticsDeviceConnector {
  pub(crate) peripheral: Peripheral,
}

impl BhapticsDeviceConnector {
  pub(crate) async fn connect(&self) -> Result<()> {
    let peripheral = &self.peripheral;

    if peripheral.is_connected().await? {
      return Err(anyhow!("Already connected"));
    }

    let properties = match peripheral.properties().await {
      Ok(Some(properties)) => properties,
      _ => return Err(anyhow!("Unable to get peripheral properties")),
    };

    peripheral.connect().await.context("Unable to connect")?;

    peripheral
      .discover_services()
      .await
      .context("Unable to discover services")?;

    // we always need serial number
    let serial_number = self.read_serial_number().await?;

    let firmware_version = self.read_firmware_version().await.ok();

    let is_senseshift = firmware_version.as_deref() == Some("255.255");

    match self.subscribe_battery_level().await {
      Ok(_) => {}
      Err(err) => warn!(
        "Unable to subscribe to battery level notifications: {:?}",
        err
      ),
    };

    info!(
      "Connected to {} bHaptics device: {:?} (SN: {:?}, Firmware: {:?})",
      if is_senseshift {
        "SenseShift"
      } else {
        "original"
      },
      properties.local_name.unwrap_or("<Unknown>".to_string()),
      serial_number,
      firmware_version.unwrap_or("<Unknown>".to_string()),
    );

    let device_task = BhapticsDeviceTask::new(peripheral.clone());

    tokio::spawn(async move {
      if let Err(err) = device_task.run().await {
        error!("Device Task failed: {:?}", err);
      }
      warn!("Device Task exited.");
    });

    Ok(())
  }
}

impl BhapticsDeviceConnector {
  async fn subscribe_battery_level(&self) -> Result<()> {
    let char = match self
      .peripheral
      .characteristics()
      .iter()
      .find(|c| c.uuid == CHAR_BATTERY)
    {
      Some(c) => c.clone(),
      None => return Err(anyhow!("CHAR_BATTERY characteristic not present")),
    };

    self
      .peripheral
      .subscribe(&char)
      .await
      .context("Unable to subscribe to CHAR_BATTERY")
  }

  #[instrument(skip(self))]
  async fn read_firmware_version(&self) -> Result<String> {
    let char = match self
      .peripheral
      .characteristics()
      .iter()
      .find(|c| c.uuid == CHAR_VERSION)
    {
      Some(c) => c.clone(),
      None => return Err(anyhow!("CHAR_VERSION characteristic not present")),
    };

    let raw_version = self
      .peripheral
      .read(&char)
      .await
      .context("Unable to read CHAR_VERSION")?;

    // map [255, 255] to "255.255"
    Ok(
      raw_version
        .iter()
        .map(|&b| b.to_string())
        .collect::<Vec<String>>()
        .join("."),
    )
  }

  async fn read_serial_number(&self) -> Result<String> {
    let char = match self
      .peripheral
      .characteristics()
      .iter()
      .find(|c| c.uuid == CHAR_SN)
    {
      Some(c) => c.clone(),
      None => return Err(anyhow!("CHAR_SN characteristic not present")),
    };

    let raw_sn = self
      .peripheral
      .read(&char)
      .await
      .context("Unable to read CHAR_SN")?;

    // map [0x01, 0x02, 0x03, ...] to "01:02:03:..."
    Ok(
      raw_sn
        .iter()
        .map(|&b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join(":"),
    )
  }
}
