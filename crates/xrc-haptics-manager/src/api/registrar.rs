use crate::api::*;
use crate::Result;
#[cfg(any(feature = "mockall", test))]
use mockall::{automock, predicate::*};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum HapticDevice {
  Body(Vec<body::BodyHapticsActuator>),
}

pub enum HapticsRequest {
  Body(body::BodyHapticsRequest),
}

#[cfg_attr(any(feature = "mockall", test), automock)]
pub trait HapticRegistrar {
  fn register(&self, device: &HapticDevice) -> Result<mpsc::Receiver<HapticsRequest>>;
}
