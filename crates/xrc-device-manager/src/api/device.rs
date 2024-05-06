use crate::api::*;
use crate::Result;

use derivative::Derivative;

use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
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
  /// Returns the device's unique identifier.
  fn id(&self) -> &DeviceId;

  fn descriptor(&self) -> Descriptor;

  async fn properties(&self) -> Result<Option<Properties>>;
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Hash)]
pub struct GenericDevice<Descriptor, Properties>
where
  Descriptor: DeviceDescriptor,
  Properties: DeviceProperties,
{
  id: DeviceId,

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  descriptor: Arc<RwLock<Descriptor>>, // todo: use ArcSwap?

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  internal: Arc<dyn DeviceInternal<Properties>>,
}

impl<Descriptor, Properties> GenericDevice<Descriptor, Properties>
where
  Descriptor: DeviceDescriptor,
  Properties: DeviceProperties,
{
  #[inline(always)]
  pub fn new(
    id: DeviceId,
    descriptor: Arc<RwLock<Descriptor>>,
    internal: Arc<dyn DeviceInternal<Properties>>,
  ) -> Self {
    Self {
      id,
      descriptor,
      internal,
    }
  }
}

#[async_trait]
impl<Descriptor, Properties> Device<Descriptor, Properties>
  for GenericDevice<Descriptor, Properties>
where
  Descriptor: DeviceDescriptor + Send + Sync + Clone,
  Properties: DeviceProperties + Send + Sync,
{
  #[inline(always)]
  fn id(&self) -> &DeviceId {
    &self.id
  }

  #[inline(always)]
  fn descriptor(&self) -> Descriptor {
    self.descriptor.read().unwrap().clone()
  }

  #[inline(always)]
  async fn properties(&self) -> Result<Option<Properties>> {
    self.internal.properties().await
  }
}
