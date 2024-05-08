mod device;
mod device_descriptor;
mod device_properties;
mod simple_device;
mod transport;

pub use device::*;
pub use device_descriptor::*;
pub use device_properties::*;
pub use simple_device::*;
pub use transport::*;

#[derive(Clone, Debug)]
pub enum DeviceManagerEvent {
  DeviceDiscovered(ConcurrentDevice),
  DeviceUpdated(ConcurrentDevice),
  DeviceConnected(ConcurrentDevice),
  DeviceDisconnected(DeviceId),
}
