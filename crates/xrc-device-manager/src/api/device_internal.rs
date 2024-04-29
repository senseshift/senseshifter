use std::fmt::Debug;

use crate::api::*;
use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait DeviceInternal<Properties>: Debug + Send + Sync
where
  Properties: DeviceProperties + Send + Sync,
{
  async fn properties(&self) -> Result<Option<Properties>>;

  async fn connect(&self) -> Result<()>;
}
