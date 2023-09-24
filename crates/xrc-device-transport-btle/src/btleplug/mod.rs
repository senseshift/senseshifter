use std::fmt::Debug;
use anyhow::Result;

use btleplug::{
  api::{
    Peripheral as ApiPeripheral,
    PeripheralProperties
  },
  platform::Peripheral
};
use btleplug::platform::PeripheralId;
use getset::Getters;
use tracing::error;

use xrc_transport::api::{Device, TransportConnector};

mod manager;
pub use manager::{
  BtlePlugManager, BtlePlugManagerBuilder,
};

mod specifier;
pub use specifier::{
  BtlePlugProtocolSpecifierBuilder, BtlePlugProtocolSpecifier,
};

#[derive(Getters, Debug, Clone)]
pub struct BtlePlugPeripheralInfo {
  #[get = "pub"]
  peripheral_id: PeripheralId,
  #[get = "pub"]
  properties: Option<PeripheralProperties>,
}

impl PartialEq for BtlePlugPeripheralInfo {
  fn eq(&self, other: &Self) -> bool {
    self.peripheral_id == other.peripheral_id
  }
}

impl Eq for BtlePlugPeripheralInfo {}

#[derive(Getters, Debug, Clone)]
pub struct BtlePlugConnector<T: ApiPeripheral + 'static> {
  #[get = "pub"]
  peripheral: T,
  #[get = "pub"]
  peripheral_info: BtlePlugPeripheralInfo,
}

pub type PlatformBtlePlugConnector = BtlePlugConnector<Peripheral>;

impl<T: ApiPeripheral> BtlePlugConnector<T> {
  pub fn new(
    peripheral: T,
    peripheral_info: BtlePlugPeripheralInfo
  ) -> Self {
    Self {
      peripheral,
      peripheral_info,
    }
  }

  pub async fn update(&mut self) {
    let properties = match self.peripheral.properties().await {
      Ok(properties) => properties,
      Err(err) => {
        error!("Unable to update peripheral properties: {}", err);
        None
      }
    };
    self.peripheral_info = BtlePlugPeripheralInfo {
     peripheral_id: self.peripheral.id(),
     properties
    };
  }
}

#[xrc_transport::async_trait]
impl<T: ApiPeripheral> TransportConnector for BtlePlugConnector<T> {
  async fn connect(&mut self) -> Result<()> {
    Err(anyhow::anyhow!("Not implemented"))
  }
}
