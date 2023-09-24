use bytes::Bytes;
use apple_continuity::{
  ContinuityMessage, ProximityPairing, ProximityPairingDeviceModel
};

use test_case::test_case;

// todo: copy from here: https://github.com/ECTO-1A/AppleJuice/blob/main/ESP32-Arduino/applejuice/applejuice.ino

#[test_case(
  &[7, 25, 1, 20, 32, 2, 240, 143, 17, 0, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  ContinuityMessage::ProximityPairing(ProximityPairing {
    device_model: ProximityPairingDeviceModel::AirPodsPro2,
    // left_battery: 0,
    // right_battery: 15,
    // case_battery: 15,
    // left_charging: false,
    // right_charging: false,
    // case_charging: false
  })
  ; "AirPods Pro 2 - Real"
)]
#[test_case(
  &[0x07, 0x19, 0x07, 0x02, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00, 0x00, 0x45, 20, 30, 150, 0xda, 0x29, 0x58, 0xab, 0x8d, 0x29, 0x40, 0x3d, 0x5c, 0x1b, 0x93, 0x3a],
  ContinuityMessage::ProximityPairing(ProximityPairing {
    device_model: ProximityPairingDeviceModel::AirPods,
    // left_battery: 10,
    // right_battery: 10,
    // case_battery: 0,
    // left_charging: true,
    // right_charging: true,
    // case_charging: false
  })
  ; "AirPods"
)]
fn proximity_pairing(bytes: &[u8], expected: ContinuityMessage) {
  let bytes = Bytes::from(bytes.to_vec());

  assert_eq!(
    ContinuityMessage::try_from(bytes).unwrap(),
    expected
  );
}