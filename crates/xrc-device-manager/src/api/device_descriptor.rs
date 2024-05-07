use crate::api::*;
use derivative::Derivative;
use std::fmt::Debug;

pub trait DeviceDescriptor: Debug {
  fn name(&self) -> &str;

  fn manufacturer(&self) -> Option<&str>;

  fn product(&self) -> Option<&str>;

  fn serial_number(&self) -> Option<&str>;
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Derivative, Debug, Clone, Eq)]
#[derivative(PartialEq, Hash)]
pub struct GenericDeviceDescriptor {
  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  name: String,

  manufacturer: Option<String>,

  product: Option<String>,

  serial_number: Option<String>,
}

impl GenericDeviceDescriptor {
  #[inline(always)]
  pub fn new(
    name: String,
    manufacturer: Option<String>,
    product: Option<String>,
    serial_number: Option<String>,
  ) -> Self {
    Self {
      name,
      manufacturer,
      product,
      serial_number,
    }
  }
}

impl DeviceDescriptor for GenericDeviceDescriptor {
  #[inline(always)]
  fn name(&self) -> &str {
    &self.name
  }

  #[inline(always)]
  fn manufacturer(&self) -> Option<&str> {
    self.manufacturer.as_deref()
  }

  #[inline(always)]
  fn product(&self) -> Option<&str> {
    self.product.as_deref()
  }

  #[inline(always)]
  fn serial_number(&self) -> Option<&str> {
    self.serial_number.as_deref()
  }
}
