use crate::Result;
use anyhow::anyhow;
use derivative::Derivative;
use dyn_clone::DynClone;
use std::fmt::Debug;
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

#[async_trait::async_trait]
pub trait Device: Send + Sync + DynClone + Debug {
  fn id(&self) -> DeviceId;

  fn name(&self) -> String;

  fn connectible(&self) -> bool {
    false
  }

  fn connected(&self) -> bool {
    false
  }

  async fn connect(&self) -> Result<()> {
    Err(anyhow!("Cannot connect to this device"))
  }
}

dyn_clone::clone_trait_object!(Device);
