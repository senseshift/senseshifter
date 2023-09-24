use anyhow::{
  Result, anyhow,
};
use bytes::Buf;

use strum::{Display, FromRepr};
use num_enum::{
  IntoPrimitive, TryFromPrimitive,
};


#[derive(Debug, Copy, Clone, Eq, PartialEq, FromRepr, Display, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum ProximityPairingDeviceModel {
  Unknown = 0,

  #[strum(serialize = "AirPods")]
  AirPods = 0x2002,

  #[strum(serialize = "AirPods 2")]
  AirPods2 = 0x200F,

  #[strum(serialize = "AirPods 3")]
  AirPods3 = 0x2013,

  #[strum(serialize = "AirPods Pro")]
  AirPodsPro = 0x200E,

  #[strum(serialize = "AirPods Pro 2")]
  AirPodsPro2 = 0x2014,

  #[strum(serialize = "AirPods Max")]
  AirPodsMax = 0x200A,

  #[strum(serialize = "PowerBeats 3")]
  PowerBeats3 = 0x2003,

  #[strum(serialize = "Beats X")]
  BeatsX = 0x2005,

  #[strum(serialize = "Beats Solo 3")]
  BeatsSolo3 = 0x2006,

  #[strum(serialize = "Beats Fit 3")]
  BeatsFit3 = 0x2012,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ProximityPairing {
  pub device_model: ProximityPairingDeviceModel,

  // pub left_battery: u8,
  // pub right_battery: u8,
  // pub case_battery: u8,
  //
  // pub left_charging: bool,
  // pub right_charging: bool,
  // pub case_charging: bool,
}

impl Default for ProximityPairing {
  fn default() -> Self {
    Self {
      device_model: ProximityPairingDeviceModel::Unknown,

      // right_battery: 0,
      // left_battery: 0,
      // case_battery: 0,
      //
      // right_charging: false,
      // left_charging: false,
      // case_charging: false,
    }
  }
}

impl TryFrom<bytes::Bytes> for ProximityPairing {
  type Error = anyhow::Error;

  /// See: https://github.com/furiousMAC/continuity/blob/master/messages/proximity_pairing.md
  fn try_from(bytes: bytes::Bytes) -> Result<Self, Self::Error> {
    let message_type = bytes[0];
    if message_type != 0x07 {
      return Err(anyhow!("Invalid message type: {}", message_type));
    }

    let message_length = bytes[1];
    if message_length != bytes.len() as u8 - 2 {
      return Err(anyhow!("Invalid message length: {}", message_length));
    }

    // Byte 2: Prefix
    let prefix = bytes[2];

    // Byte 3-4: Device model
    let device_model = bytes.slice(3..5).get_u16_le();
    let device_model = ProximityPairingDeviceModel::try_from(device_model).unwrap_or(ProximityPairingDeviceModel::Unknown);

    // // Byte 5: Status
    // // todo: copy from here: https://github.com/furiousMAC/continuity/blob/master/dissector/3.4.4/packet-bthci_cmd.c#L1328
    // let status = bytes[5];
    //
    // // Byte 6: Battery levels for right (first 4 bits) and left (last 4 bits)
    // let right_battery = bytes[6] >> 4;
    // let left_battery = bytes[6] & 0b0000_1111;
    //
    // // Byte 7: Battery status and case level
    // let _ = bytes[7] & 0b1000_0000 != 0;
    // let case_charging = bytes[7] & 0b0100_0000 != 0;
    // let right_charging = bytes[7] & 0b0010_0000 != 0;
    // let left_charging = bytes[7] & 0b0001_0000 != 0;
    // let case_battery = bytes[7] & 0b0000_1111;
    //
    // // Byte 8: Lid Open Counter
    // let lid_counter = bytes[8];
    //
    // // Byte 9: Device Color
    // // todo: copy from here: https://github.com/furiousMAC/continuity/blob/master/dissector/3.4.4/packet-bthci_cmd.c#L1360
    // let device_color = bytes[9];
    //
    // // Byte 10: Suffix
    // let suffix = bytes[10];
    //
    // // Byte 11+: Encrypted data
    // let encrypted_data = bytes.slice(11..);

    // // Based on here: https://github.com/hexway/apple_bleee/blob/master/adv_airpods.py
    // // Byte 12: Battery levels for right
    // let right_battery = bytes[12];
    // // Byte 13: Battery levels for left
    // let left_battery = bytes[13];


    Ok(Self {
      device_model,

      // right_battery: 0,
      // left_battery: 0,
      // case_battery: 0,
      //
      // right_charging: false,
      // left_charging: false,
      // case_charging: false,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::convert::{
    TryInto, TryFrom,
  };
}