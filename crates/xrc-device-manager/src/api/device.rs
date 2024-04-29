use crate::api::*;
use crate::Result;

use derivative::Derivative;

use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative, Clone, PartialEq, Eq, Hash)]
#[derivative(Debug = "transparent")]
pub struct DeviceId(#[cfg_attr(feature = "serde", serde(with = "uuid::serde::urn"))] Uuid);

impl DeviceId {
  #[inline(always)]
  pub fn new(uuid: Uuid) -> Self {
    Self(uuid)
  }
}

#[async_trait]
pub trait Device<Descriptor, Properties>
where
  Descriptor: DeviceDescriptor,
  Properties: DeviceProperties,
{
  fn descriptor(&self) -> &Descriptor;

  async fn properties(&self) -> Result<Option<Properties>>;

  async fn connect(&self) -> Result<()>;
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Hash)]
pub struct GenericDevice {
  descriptor: GenericDeviceDescriptor,

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  internal: Arc<dyn DeviceInternal<GenericDeviceProperties>>,
}

impl GenericDevice {
  #[inline(always)]
  pub fn new(
    descriptor: GenericDeviceDescriptor,
    internal: Arc<dyn DeviceInternal<GenericDeviceProperties>>,
  ) -> Self {
    Self {
      descriptor,
      internal,
    }
  }
}

#[async_trait]
impl Device<GenericDeviceDescriptor, GenericDeviceProperties> for GenericDevice {
  #[inline(always)]
  fn descriptor(&self) -> &GenericDeviceDescriptor {
    &self.descriptor
  }

  #[inline(always)]
  async fn properties(&self) -> Result<Option<GenericDeviceProperties>> {
    self.internal.properties().await
  }

  #[inline(always)]
  async fn connect(&self) -> Result<()> {
    self.internal.connect().await
  }
}
