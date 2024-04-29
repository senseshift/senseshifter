use crate::Result;
use btleplug::api::{Peripheral as _, ValueNotification};
use btleplug::platform::Peripheral;

use futures::StreamExt;

use anyhow::Context;
use tracing::{info, instrument};

pub(crate) struct BhapticsDeviceTask {
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
        info!("Received battery level notification: {:?}", event);
      }
      _ => {
        info!("Received notification: {:?}", event);
      }
    }
  }
}
