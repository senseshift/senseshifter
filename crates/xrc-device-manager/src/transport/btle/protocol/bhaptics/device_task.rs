use crate::Result;
use btleplug::api::{Peripheral as _, ValueNotification, WriteType};
use btleplug::platform::Peripheral;

use futures::StreamExt;

use anyhow::{anyhow, Context};
use tracing::info;
use uuid::Uuid;

pub const CHAR_MOTOR_WRITE_STABLE: Uuid = Uuid::from_u128(0x6e40000a_b5a3_f393_e0a9_e50e24dcca9e);

pub(super) struct BhapticsDeviceTask {
  peripheral: Peripheral,
}

impl BhapticsDeviceTask {
  pub fn new(peripheral: Peripheral) -> Self {
    Self { peripheral }
  }

  pub async fn run(self) -> Result<()> {
    let mut notification_stream = self
      .peripheral
      .notifications()
      .await
      .context("Unable to get notifications stream")?;

    let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
      tokio::select! {
        notification = notification_stream.next() => {
          match notification {
            Some(notification) => self.handle_notification(notification),
            None => info!("Notification stream ended."),
          }
        },
        _ = ping_interval.tick() => {
          match self.ping().await {
            Ok(_) => info!("Ping successful"),
            Err(e) => info!("Ping failed: {:?}", e),
          }
        }
      }
    }

    Ok(())
  }

  fn handle_notification(&self, event: ValueNotification) {
    info!("Notification: {:?}", event);
  }

  async fn ping(&self) -> Result<()> {
    let motor_char = match self
      .peripheral
      .characteristics()
      .iter()
      .find(|c| c.uuid == CHAR_MOTOR_WRITE_STABLE)
    {
      Some(c) => c.clone(),
      None => return Err(anyhow!("Motor characteristic not present")),
    };

    self
      .peripheral
      .write(
        &motor_char,
        &[255, 255, 255, 255, 255, 255, 255, 255],
        WriteType::WithoutResponse,
      )
      .await?;

    // sleep 0.5s
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    self
      .peripheral
      .write(
        &motor_char,
        &[0, 0, 0, 0, 0, 0, 0, 0],
        WriteType::WithoutResponse,
      )
      .await?;

    Ok(())
  }
}
