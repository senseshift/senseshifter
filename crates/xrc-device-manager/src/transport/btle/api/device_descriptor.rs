use crate::transport::btle::api::DeviceId;

pub trait DeviceDescriptor {
  fn id(&self) -> &DeviceId;

  fn name(&self) -> &String;
}

pub struct GenericDeviceDescriptor {}
