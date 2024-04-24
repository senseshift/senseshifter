use anyhow::{anyhow, Context};
use async_trait::async_trait;
use btleplug::api::{Peripheral as _, PeripheralProperties};
use btleplug::platform::Peripheral;
use futures::{pin_mut, StreamExt};
use tracing::{error, info, instrument, warn};
use crate::Result;
use crate::transport::btle::api::*;
use uuid::Uuid;

pub const CHAR_BATTERY: Uuid = Uuid::from_u128(0x6e400008_b5a3_f393_e0a9_e50e24dcca9e);

#[derive(Debug, Clone)]
pub(super) struct BhapticsDevice {
  peripheral: Peripheral,
  name: String,
}

#[async_trait::async_trait]
impl Device for BhapticsDevice {
  fn id(&self) -> String {
    self.peripheral.id().to_string()
  }

  fn name(&self) -> String {
    self.name.clone()
  }

  fn connectible(&self) -> bool {
    true
  }

  async fn connect(&self) -> Result<()> {
    if self.peripheral.is_connected().await? {
      warn!("Peripheral already connected");
      return Ok(());
    }

    self.peripheral.connect().await.context("Unable to connect")?;

    self.peripheral.discover_services().await.context("Unable to discover services")?;

    // TODO: do not abort connection, just warn
    let battery_char = match self.peripheral
      .characteristics()
      .iter()
      .find(|c| c.uuid == CHAR_BATTERY) {
      Some(c) => c.clone(),
      None => return Err(anyhow!("Battery characteristic not present"))
    };

    match self.peripheral.subscribe(&battery_char).await {
      Ok(_) => info!("Subscribed to battery characteristic"),
      Err(err) => error!("Error subscribing to battery characteristic: {:?}", err),
    }

    let mut stream = self.peripheral.notifications().await?;

    tokio::spawn(async move {
      while let Some(event) = stream.next().await {
        info!("Event: {:?}", event);
      }
    });

    Ok(())
  }
}

#[derive(Default)]
pub struct BhapticsProtocolHandlerBuilder {}

impl BtlePlugProtocolHandlerBuilder for BhapticsProtocolHandlerBuilder {
  fn finish(&self) -> Box<dyn BtlePlugProtocolHandler> {
    Box::new(BhapticsProtocolHandler {})
  }
}

pub struct BhapticsProtocolHandler {}

#[async_trait::async_trait]
impl BtlePlugProtocolHandler for BhapticsProtocolHandler {
  fn name(&self) -> &'static str {
    "bhaptics"
  }

  #[instrument(skip(self, peripheral))]
  fn specify_protocol(&self, peripheral: Peripheral, properties: Option<PeripheralProperties>) -> Result<Option<Box<dyn Device>>> {
    let properties = match properties {
      Some(properties) => properties,
      None => return Ok(None),
    };

    let name = match properties.local_name {
      Some(ref name) => name.clone(),
      None => return Ok(None),
    };

    let appearance = match properties.appearance {
      Some(appearance) => appearance,
      None => return Ok(None),
    };

    if appearance == 508 {
      return Ok(Some(Box::new(BhapticsDevice {
        peripheral,
        name,
      })));
    }

    Ok(None)
  }
}
