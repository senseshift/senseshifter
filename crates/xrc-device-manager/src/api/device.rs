use crate::Result;
use anyhow::anyhow;
use dyn_clone::DynClone;
use std::fmt::Debug;

// todo: use a more specific type for device id? Uuid?
pub type DeviceId = String;

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
