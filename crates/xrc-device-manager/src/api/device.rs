use crate::api::*;
use crate::Result;

use derivative::Derivative;

use async_trait::async_trait;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use uuid::Uuid;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative, Copy, Clone, PartialEq, Eq, Hash)]
#[derivative(Debug = "transparent")]
pub struct DeviceId(#[cfg_attr(feature = "serde", serde(with = "uuid::serde::urn"))] Uuid);

impl DeviceId {
  #[inline(always)]
  pub fn new(uuid: Uuid) -> Self {
    Self(uuid)
  }
}

impl Display for DeviceId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[async_trait]
pub trait Device<Descriptor, Properties>: Debug + Send + Sync
where
  Descriptor: DeviceDescriptor,
  Properties: DeviceProperties,
{
  /// Returns the device's unique identifier.
  fn id(&self) -> &DeviceId;

  fn descriptor(&self) -> Descriptor;

  async fn properties(&self) -> Result<Option<Properties>>;

  fn connectible(&self) -> bool {
    false
  }

  async fn is_connected(&self) -> bool {
    false
  }

  async fn connect(&self) -> Result<()>;
}

/// Thread-safe device handle
pub type ConcurrentDevice = Arc<dyn Device<GenericDeviceDescriptor, GenericDeviceProperties>>;
