mod proximity_pairing;
pub use proximity_pairing::*;

use anyhow::anyhow;
use strum::{FromRepr, EnumDiscriminants};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use bytes::Bytes;

/// See: https://github.com/furiousMAC/continuity/blob/master/dissector/FIELDS.md
/// See: https://petsymposium.org/2020/files/papers/issue1/popets-2020-0003.pdf
/// See: https://arxiv.org/pdf/1904.10600.pdf
#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(FromRepr, IntoPrimitive, TryFromPrimitive))]
#[repr(u8)]
pub enum ContinuityMessage {
  AirPrint = 0x03,
  AirDrop = 0x05,
  HomeKit = 0x06,
  ProximityPairing(ProximityPairing) = 0x07,
  HeySiri = 0x08,
  AirPlayTarget = 0x09,
  AirPlaySource = 0x0a,
  MagicSwitch = 0x0b,
  Handoff = 0x0c,
  TetheringTarget = 0x0d,
  TetheringSource = 0x0e,
  NearbyAction = 0x0f,
  NearbyInfo = 0x10,
  FindMy = 0x12,
}

impl TryFrom<Bytes> for ContinuityMessage {
  type Error = anyhow::Error;

  fn try_from(value: Bytes) -> Result<Self, Self::Error> {
    let message_type = value[0];

    let message_length = value[1];
    let message = value.slice(2..);
    if message_length != message.len() as u8 {
      return Err(anyhow!("Message length mismatch: {} != {}", message_length, message.len()));
    }

    match ContinuityMessageDiscriminants::try_from(message_type)? {
      ContinuityMessageDiscriminants::AirPrint => Ok(ContinuityMessage::AirPrint),
      ContinuityMessageDiscriminants::AirDrop => Ok(ContinuityMessage::AirDrop),
      ContinuityMessageDiscriminants::HomeKit => Ok(ContinuityMessage::HomeKit),
      ContinuityMessageDiscriminants::ProximityPairing => {
        Ok(ContinuityMessage::ProximityPairing(ProximityPairing::try_from(value)?))
      }
      ContinuityMessageDiscriminants::HeySiri => Ok(ContinuityMessage::HeySiri),
      ContinuityMessageDiscriminants::AirPlayTarget => Ok(ContinuityMessage::AirPlayTarget),
      ContinuityMessageDiscriminants::AirPlaySource => Ok(ContinuityMessage::AirPlaySource),
      ContinuityMessageDiscriminants::MagicSwitch => Ok(ContinuityMessage::MagicSwitch),
      ContinuityMessageDiscriminants::Handoff => Ok(ContinuityMessage::Handoff),
      ContinuityMessageDiscriminants::TetheringTarget => Ok(ContinuityMessage::TetheringTarget),
      ContinuityMessageDiscriminants::TetheringSource => Ok(ContinuityMessage::TetheringSource),
      ContinuityMessageDiscriminants::NearbyAction => Ok(ContinuityMessage::NearbyAction),
      ContinuityMessageDiscriminants::NearbyInfo => Ok(ContinuityMessage::NearbyInfo),
      ContinuityMessageDiscriminants::FindMy => Ok(ContinuityMessage::FindMy),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_from_repr() {
    assert_eq!(
      ContinuityMessageDiscriminants::from_repr(0x7).unwrap(),
      ContinuityMessageDiscriminants::ProximityPairing
    );
  }
}
