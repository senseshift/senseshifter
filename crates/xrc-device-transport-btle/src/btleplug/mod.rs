use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
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

mod device;
pub use device::*;

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
  #[get = "pub"]
  connected: Arc<AtomicBool>,
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
      connected: Arc::new(AtomicBool::new(false)),
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

    self.peripheral_info.properties = properties;
  }
}

#[xrc_transport::async_trait]
impl<T: ApiPeripheral> TransportConnector for BtlePlugConnector<T> {
  async fn connect(&mut self) -> Result<()> {
    self.peripheral.connect().await.map_err(|err| err.into())
  }

  async fn disconnect(&mut self) -> Result<()> {
    self.peripheral.disconnect().await.map_err(|err| err.into())
  }

  fn is_connected(&self) -> bool {
    self.connected.load(std::sync::atomic::Ordering::SeqCst)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeSet;
  use std::pin::Pin;
  use btleplug::api::{BDAddr, Characteristic, Service, ValueNotification, WriteType};
  use futures::Stream;
  use super::*;

  #[test]
  fn shared_state() {
    let connector = BtlePlugConnector::new(
      TestPeripheral {},
      BtlePlugPeripheralInfo {
        peripheral_id: PeripheralId::from(BDAddr::from([0; 6])),
        properties: None,
      }
    );

    let connector_clone = connector.clone();
    connector.connected.store(true, std::sync::atomic::Ordering::SeqCst);

    assert_eq!(connector_clone.is_connected(), true);
  }

  #[derive(Debug, Clone)]
  struct TestPeripheral {
  }

  #[xrc_transport::async_trait]
  impl btleplug::api::Peripheral for TestPeripheral {
    fn id(&self) -> PeripheralId {
      todo!()
    }

    fn address(&self) -> BDAddr {
      todo!()
    }

    async fn properties(&self) -> btleplug::Result<Option<PeripheralProperties>> {
      todo!()
    }

    fn services(&self) -> BTreeSet<Service> {
      todo!()
    }

    async fn is_connected(&self) -> btleplug::Result<bool> {
      todo!()
    }

    async fn connect(&self) -> btleplug::Result<()> {
      todo!()
    }

    async fn disconnect(&self) -> btleplug::Result<()> {
      todo!()
    }

    async fn discover_services(&self) -> btleplug::Result<()> {
      todo!()
    }

    async fn write(&self, characteristic: &Characteristic, data: &[u8], write_type: WriteType) -> btleplug::Result<()> {
      todo!()
    }

    async fn read(&self, characteristic: &Characteristic) -> btleplug::Result<Vec<u8>> {
      todo!()
    }

    async fn subscribe(&self, characteristic: &Characteristic) -> btleplug::Result<()> {
      todo!()
    }

    async fn unsubscribe(&self, characteristic: &Characteristic) -> btleplug::Result<()> {
      todo!()
    }

    async fn notifications(&self) -> btleplug::Result<Pin<Box<dyn Stream<Item=ValueNotification> + Send>>> {
      todo!()
    }
  }
}
