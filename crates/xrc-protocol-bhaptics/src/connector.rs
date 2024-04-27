use crate::constants::CHAR_BATTERY;
use crate::device_task::BhapticsDeviceTask;
use crate::Result;
use anyhow::{anyhow, Context};
use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub(crate) struct BhapticsDeviceConnector {
  pub(crate) peripheral: Peripheral,
}

impl BhapticsDeviceConnector {
  pub(crate) async fn connect(&self) -> Result<()> {
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
