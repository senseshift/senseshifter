use crate::Result;
use btleplug::api::{Peripheral as _, ValueNotification};
use btleplug::platform::Peripheral;
use std::sync::{Arc, RwLock};

use futures::StreamExt;

use anyhow::Context;
use tracing::{info, instrument};
use xrc_device_transport_btleplug::api::DeviceBatteryProperty;

pub(crate) struct BhapticsDeviceTask {
  peripheral: Peripheral,
  battery_level: Arc<RwLock<Option<DeviceBatteryProperty>>>,
}

impl BhapticsDeviceTask {
  pub fn new(
    peripheral: Peripheral,
    battery_level: Arc<RwLock<Option<DeviceBatteryProperty>>>,
  ) -> Self {
    Self {
      peripheral,
      battery_level,
    }
  }

  pub async fn run(self) -> Result<()> {
    // Subscribe to battery level notifications
    let battery_char = self
      .peripheral
      .characteristics()
      .into_iter()
      .find(|c| c.uuid == super::constants::CHAR_BATTERY);
    if let Some(battery_char) = battery_char {
      if let Err(err) = self.peripheral.subscribe(&battery_char).await {
        info!(
          "Failed to subscribe to battery level notifications: {:?}",
          err
        );
      }
    }

    let mut notification_stream = self
      .peripheral
      .notifications()
      .await
      .context("Unable to get notifications stream")?;

    loop {
      tokio::select! {
        notification = notification_stream.next() => {
          match notification {
            Some(notification) => self.handle_notification(notification),
            None => info!("Notification stream ended."),
          }
        }
      }
    }
  }

  #[instrument(skip(self))]
  fn handle_notification(&self, event: ValueNotification) {
    match event.uuid {
      uuid if uuid == super::constants::CHAR_BATTERY => {
        let level = event.value[0];
        let level = level as f32 / 100.0;

        let mut battery = DeviceBatteryProperty::default();
        battery.set_level(level);

        match self.battery_level.write() {
          Ok(mut guard) => {
            guard.replace(battery);
          }
          Err(err) => {
            info!("Failed to update battery level: {:?}", err);
          }
        }
      }
      _ => {
        info!("Received notification: {:?}", event);
      }
    }
  }
}
