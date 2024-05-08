use crate::api::*;
use crate::Result;
use async_trait::async_trait;
use btleplug::platform::Peripheral;
use derivative::Derivative;
use std::fmt::Debug;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct BtlePlugDevice {
  id: DeviceId,

  #[derivative(Debug = "ignore")]
  peripheral: Peripheral,

  internal: Box<dyn BtlePlugDeviceInternal>,
}

impl BtlePlugDevice {
  pub fn new(
    id: DeviceId,
    peripheral: Peripheral,
    internal: Box<dyn BtlePlugDeviceInternal>,
  ) -> Self {
    Self {
      id,
      peripheral,
      internal,
    }
  }
}

#[async_trait]
impl Device<GenericDeviceDescriptor, GenericDeviceProperties> for BtlePlugDevice {
  fn id(&self) -> &DeviceId {
    &self.id
  }

  fn descriptor(&self) -> GenericDeviceDescriptor {
    self.internal.descriptor()
  }

  async fn properties(&self) -> Result<Option<GenericDeviceProperties>> {
    self.internal.properties().await
  }

  fn connectible(&self) -> bool {
    self.internal.connectible()
  }

  async fn connect(&self) -> Result<()> {
    self.internal.connect().await
  }
}

#[async_trait]
pub trait BtlePlugDeviceInternal: Send + Sync + Debug {
  fn descriptor(&self) -> GenericDeviceDescriptor;

  async fn properties(&self) -> Result<Option<GenericDeviceProperties>>;

  fn connectible(&self) -> bool {
    false
  }

  async fn connect(&self) -> Result<()>;
}
