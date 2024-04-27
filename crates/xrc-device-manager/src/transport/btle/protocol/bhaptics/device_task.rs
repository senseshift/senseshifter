use btleplug::api::ValueNotification;
use btleplug::platform::Peripheral;
use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;
use tracing::info;

pub(super) struct BhapticsDeviceTask {
  peripheral: Peripheral,
  notification_stream: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
}

impl BhapticsDeviceTask {
  pub fn new(
    peripheral: Peripheral,
    notification_stream: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
  ) -> Self {
    Self {
      peripheral,
      notification_stream,
    }
  }

  pub async fn run(mut self) {
    while let Some(notification) = self.notification_stream.next().await {
      self.handle_notification(notification);
    }
  }

  fn handle_notification(&self, event: ValueNotification) {
    info!("Notification: {:?}", event);
  }
}
