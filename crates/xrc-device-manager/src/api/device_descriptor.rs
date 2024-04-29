use crate::api::*;
use derivative::Derivative;

pub trait DeviceDescriptor {
  fn id(&self) -> &DeviceId;

  fn name(&self) -> &str;
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative, Debug, Clone, Eq)]
#[derivative(PartialEq, Hash)]
pub struct GenericDeviceDescriptor {
  id: DeviceId,

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  name: String,
}

impl GenericDeviceDescriptor {
  #[inline(always)]
  pub fn new(id: DeviceId, name: impl Into<String>) -> Self {
    Self {
      id,
      name: name.into(),
    }
  }
}

impl DeviceDescriptor for GenericDeviceDescriptor {
  #[inline(always)]
  fn id(&self) -> &DeviceId {
    &self.id
  }

  #[inline(always)]
  fn name(&self) -> &str {
    &self.name
  }
}

impl From<GenericDeviceDescriptor> for DeviceId {
  #[inline(always)]
  fn from(val: GenericDeviceDescriptor) -> Self {
    val.id
  }
}
