use std::fmt::Debug;
use anyhow::anyhow;
use dyn_clone::DynClone;
use crate::Result;

#[async_trait::async_trait]
pub trait Device: Send + Sync + DynClone + Debug {
  fn id(&self) -> String;

  fn name(&self) -> String;

  fn connectible(&self) -> bool {
    false
  }

  async fn connect(&self) -> Result<()> {
    Err(anyhow!("Cannot connect to this device"))
  }
}

dyn_clone::clone_trait_object!(Device);