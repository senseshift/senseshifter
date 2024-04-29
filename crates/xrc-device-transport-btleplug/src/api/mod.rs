use btleplug::api::BDAddr;
use uuid::Uuid;
pub use xrc_device_manager::api::*;

mod protocol;

pub use protocol::*;

/// Convert a Bluetooth address to a device ID.
pub fn address_to_id(address: &BDAddr) -> DeviceId {
  let inner: [u8; 6] = address.into_inner();

  let inner: [u8; 16] = [
    0xc0, 0x01, 0xfa, 0xce, 0xba, 0xbe, 0xff, 0xff, 0xb7, 0x1e, inner[0], inner[1], inner[2],
    inner[3], inner[4], inner[5],
  ];
  let uuid = Uuid::new_v8(inner);

  DeviceId::new(uuid)
}
