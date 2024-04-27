mod constants;
mod device_config;
mod device_task;
mod protocol_specifier;

use constants::*;
pub use protocol_specifier::*;

use anyhow::{anyhow, Context};

use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use xrc_device_transport_btleplug::api::*;

use crate::device_task::BhapticsDeviceTask;
use tracing::{error, info, instrument, warn};

pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, Clone)]
pub(crate) struct BhapticsDevice {
  peripheral: Peripheral,
  name: String,
}

#[async_trait::async_trait]
impl Device for BhapticsDevice {
  fn id(&self) -> DeviceId {
    self.peripheral.id().to_string()
  }

  fn name(&self) -> String {
    self.name.clone()
  }

  fn connectible(&self) -> bool {
    true
  }

  #[instrument(skip(self))]
  async fn connect(&self) -> Result<()> {
    if self.peripheral.is_connected().await? {
      return Err(anyhow!("Already connected"));
    }

    self
      .peripheral
      .connect()
      .await
      .context("Unable to connect")?;

    self
      .peripheral
      .discover_services()
      .await
      .context("Unable to discover services")?;

    // TODO: do not abort connection, just warn
    let battery_char = match self
      .peripheral
      .characteristics()
      .iter()
      .find(|c| c.uuid == CHAR_BATTERY)
    {
      Some(c) => c.clone(),
      None => return Err(anyhow!("Battery characteristic not present")),
    };

    match self.peripheral.subscribe(&battery_char).await {
      Ok(_) => info!("Subscribed to battery characteristic"),
      Err(err) => error!("Error subscribing to battery characteristic: {:?}", err),
    }

    let device_task = BhapticsDeviceTask::new(self.peripheral.clone());

    tokio::spawn(async move {
      if let Err(err) = device_task.run().await {
        error!("Device Task failed: {:?}", err);
      }
      warn!("Device Task exited.");
    });

    Ok(())
  }
}
