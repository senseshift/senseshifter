use crate::Result;
use std::fmt::Debug;

#[async_trait::async_trait]
pub trait TransportConnector: Sync + Send + Debug {
  async fn connect(&mut self) -> Result<()>
  {
    Ok(())
  }
}