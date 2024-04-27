use crate::api::*;

pub trait DeviceDescriptor {
  fn id(&self) -> &DeviceId;

  fn name(&self) -> &String;
}

pub struct GenericDeviceDescriptor {}
