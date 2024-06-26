use anyhow::anyhow;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use crate::api::*;
use crate::Result;
use async_trait::async_trait;
use derivative::Derivative;

#[cfg(any(feature = "mockall", test))]
use mockall::{automock, predicate::*};

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Hash)]
pub struct SimpleDevice {
  id: DeviceId,

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  descriptor: Arc<RwLock<GenericDeviceDescriptor>>, // todo: use ArcSwap or keepcalm?

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  connectible: bool,

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  connected: Arc<AtomicBool>,

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  internal: Arc<dyn SimpleDeviceInternal<GenericDeviceProperties>>,
}

impl SimpleDevice {
  #[inline(always)]
  pub fn new(
    id: DeviceId,
    descriptor: Arc<RwLock<GenericDeviceDescriptor>>,
    connectible: bool,
    internal: Arc<dyn SimpleDeviceInternal<GenericDeviceProperties>>,
  ) -> Self {
    Self {
      id,
      descriptor,
      connectible,
      connected: Arc::new(AtomicBool::new(false)),
      internal,
    }
  }
}

#[async_trait]
impl Device<GenericDeviceDescriptor, GenericDeviceProperties> for SimpleDevice {
  #[inline(always)]
  fn id(&self) -> &DeviceId {
    &self.id
  }

  #[inline(always)]
  fn descriptor(&self) -> GenericDeviceDescriptor {
    self.descriptor.read().unwrap().clone()
  }

  #[inline(always)]
  async fn properties(&self) -> Result<Option<GenericDeviceProperties>> {
    self.internal.properties().await
  }

  #[inline(always)]
  fn connectible(&self) -> bool {
    self.connectible
  }

  async fn connect(&self) -> Result<()> {
    if !self.connectible {
      return Err(anyhow!("Device is not connectible"));
    }

    if self.connected.load(Ordering::SeqCst) {
      return Err(anyhow!("Device is already connected"));
    }

    self.internal.connect().await?;

    self.connected.store(true, Ordering::SeqCst);

    Ok(())
  }
}

#[cfg_attr(any(feature = "mockall", test), automock)]
#[async_trait]
pub trait SimpleDeviceInternal<Properties>: Debug + Send + Sync
where
  Properties: DeviceProperties + Send + Sync,
{
  async fn properties(&self) -> Result<Option<Properties>>;

  async fn connect(&self) -> Result<()>;
}
