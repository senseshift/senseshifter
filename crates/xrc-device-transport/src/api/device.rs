use std::fmt::Debug;
use async_trait::async_trait;

#[async_trait]
pub trait Device: Send + Sync + Debug {
  fn id(&self) -> &String;

  fn name(&self) -> Option<String> {
    None
  }

  fn connectible(&self) -> bool {
    false
  }

  fn is_connected(&self) -> bool {
    false
  }

  fn connect(&self) -> crate::Result<()> {
    Err(anyhow::anyhow!("Device is not connectible"))
  }

  fn disconnect(&self) -> crate::Result<()> {
    Err(anyhow::anyhow!("Device is not connectible"))
  }
}
