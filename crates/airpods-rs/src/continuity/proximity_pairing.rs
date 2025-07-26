use anyhow::{anyhow, Result};
use bytes::Buf;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::{Display, FromRepr};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, FromRepr, Display, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum ProximityPairingDeviceModel {
    #[default]
    Unknown = 0,

    #[strum(serialize = "AirPods")]
    AirPods = 0x2002,

    #[strum(serialize = "AirPods 2")]
    AirPods2 = 0x200F,

    #[strum(serialize = "AirPods 3")]
    AirPods3 = 0x2013,

    #[strum(serialize = "AirPods 4")]
    AirPods4 = 0x1920,

    #[strum(serialize = "AirPods 4 (ANC)")]
    AirPods4Anc = 0x1B20,

    #[strum(serialize = "AirPods Pro")]
    AirPodsPro = 0x200E,

    #[strum(serialize = "AirPods Pro 2")]
    AirPodsPro2 = 0x2014,

    #[strum(serialize = "AirPods Max")]
    AirPodsMax = 0x200A,

    #[strum(serialize = "AirPods Max (USB-C)")]
    AirPodsMaxUsbC = 0x1F20,

    #[strum(serialize = "PowerBeats 3")]
    PowerBeats3 = 0x2003,

    #[strum(serialize = "Beats X")]
    BeatsX = 0x2005,

    #[strum(serialize = "Beats Solo 3")]
    BeatsSolo3 = 0x2006,

    #[strum(serialize = "Beats Fit 3")]
    BeatsFit3 = 0x2012,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, FromRepr, Display, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ProximityPairingDeviceColor {
    #[default]
    White = 0x00,
    Black = 0x01,
    Red = 0x02,
    Blue = 0x03,
    Pink = 0x04,
    Gray = 0x05,
    Silver = 0x06,
    Gold = 0x07,
    RoseGold = 0x08,
    SpaceGray = 0x09,
    DarkBlue = 0x0A,
    LightBlue = 0x0B,
    Yellow = 0x0C,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ProximityPairing {
    pub device_model: ProximityPairingDeviceModel,
    pub device_color: ProximityPairingDeviceColor,

    pub left_battery: Option<u8>,
    pub right_battery: Option<u8>,
    pub case_battery: Option<u8>,

    pub left_charging: bool,
    pub right_charging: bool,
    pub case_charging: bool,
}

impl Default for ProximityPairing {
    fn default() -> Self {
        Self {
            device_model: ProximityPairingDeviceModel::default(),
            device_color: ProximityPairingDeviceColor::default(),

            left_battery: None,
            right_battery: None,
            case_battery: None,

            left_charging: false,
            right_charging: false,
            case_charging: false,
        }
    }
}

impl TryFrom<bytes::Bytes> for ProximityPairing {
    type Error = anyhow::Error;

    /// Decoding the beacon data
    ///
    /// There are multiple subversions of the same protocol:
    /// - The first one is initially covered by the [`furiousMAC`][furiousMAC]. It uses bytes `7` and `8` to encode the battery levels.
    /// - The second one uses bytes `12` and `13` to encode the battery levels, and these bytes may be flipped. I wasn't able to find a nicely formatted source for this version, but you can find it in [`hexway`][hexway] and [`PodsCompanion`][PodsCompanion].
    ///
    /// For this parser, we'll use the second version. This may leave some devices with older firmware to behave unexpectedly, but I do not know how to distinguish between the two versions.
    ///
    /// [furiousMAC]: https://github.com/furiousMAC/continuity/blob/master/messages/proximity_pairing.md (An Apple Continuity Protocol Reverse Engineering Project)
    /// [hexway]: https://github.com/hexway/apple_bleee/blob/master/adv_airpods.py (hexway's AirPods mimic attack)
    /// [PodsCompanion]: https://github.com/Domi04151309/PodsCompanion/blob/master/app/src/main/java/io/github/domi04151309/podscompanion/services/PodsService.kt#L122 (PodsCompanion parser implementation)
    /// [OpenPods]: https://github.com/adolfintel/OpenPods/blob/master/app/src/main/java/com/dosse/airpods/pods/PodsStatus.java (OpenPods parser implementation)
    /// [librepods]: https://github.com/kavishdevar/librepods/blob/main/android/app/src/main/java/me/kavishdevar/librepods/utils/BLEManager.kt#L276
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
        let is_paired = prefix == 0x01;
        // println!("[ProximityPairing] is_paired: {}", is_paired);

        // Byte 3-4: Device model
        let device_model = bytes.slice(3..5).get_u16_le();
        let device_model = ProximityPairingDeviceModel::try_from(device_model)
            .unwrap_or_default();

        // Byte 5: Status
        let status = bytes[5];

        let primary_is_left = status & 0b0000_0001 != 0;

        let primary_in_ear = (status & 0b0000_0010) != 0;
        let both_in_case = (status & 0b0000_0100) != 0;
        let another_in_ear = (status & 0b0000_1000) != 0;

        let broadcast_from_left = (status & 0b0010_0000) != 0;

        // Byte 6: Battery levels for right (first 4 bits) and left (last 4 bits)
        // let right_battery = bytes[6] >> 4;
        // let left_battery = bytes[6] & 0b0000_1111;

        // // Byte 7: Battery status and case level
        // let _ = bytes[7] & 0b1000_0000 != 0;
        // let case_charging = bytes[7] & 0b0100_0000 != 0;
        // let right_charging = bytes[7] & 0b0010_0000 != 0;
        // let left_charging = bytes[7] & 0b0001_0000 != 0;
        // let case_battery = bytes[7] & 0b0000_1111;

        // // Byte 8: Lid Open Counter
        // let lid_counter = bytes[8];

        // Byte 9: Device Color
        let device_color = bytes[9];
        let device_color = ProximityPairingDeviceColor::try_from(device_color)
            .unwrap_or_default();

        // Byte 10: Suffix
        let suffix = bytes[10];
        let flipped: bool = suffix & 0x02 == 0;

        // Byte 12-13: Battery levels
        // (0-10 battery; 15=disconnected)
        let left_battery = if flipped { bytes[12] } else { bytes[13] };
        let right_battery = if flipped { bytes[13] } else { bytes[12] };

        // Byte 14: Charging status
        // (bit 0/1=left/right; bit 2=case)
        let charge_status = bytes[14];

        let left_charging = (if flipped {
            charge_status & 0b00000010
        } else {
            charge_status & 0b00000001
        }) != 0;
        let right_charging = (if flipped {
            charge_status & 0b00000001
        } else {
            charge_status & 0b00000010
        }) != 0;
        let single_battery_charging = charge_status & 0b00000001 != 0;
        let case_charging = charge_status & 0b00000100 != 0;

        // Byte 15: Case battery
        // (0-10 batt; 15=disconnected)
        let case_battery = bytes[15];

        // Byte 16: In Ear Detection
        let in_ear_status = bytes[16];
        let left_in_ear = (if flipped {
            in_ear_status & 0b00001000
        } else {
            in_ear_status & 0b00000010
        }) != 0;
        let right_in_ear = (if flipped {
            in_ear_status & 0b00000010
        } else {
            in_ear_status & 0b00001000
        }) != 0;

        const DISCONNECTED_STATUS: u8 = 15;
        const MAX_CONNECTED_STATUS: u8 = 10;
        const LOW_BATTERY_STATUS: u8 = 1;

        let left_battery: Option<_> = if left_battery == MAX_CONNECTED_STATUS {
            Some(100)
        } else if left_battery < MAX_CONNECTED_STATUS {
            Some(left_battery * 10 + 5)
        } else {
            None
        };
        let right_battery: Option<_> = if right_battery == MAX_CONNECTED_STATUS {
            Some(100)
        } else if right_battery < MAX_CONNECTED_STATUS {
            Some(right_battery * 10 + 5)
        } else {
            None
        };
        let case_battery: Option<_> = if case_battery == MAX_CONNECTED_STATUS {
            Some(100)
        } else if case_battery < MAX_CONNECTED_STATUS {
            Some(case_battery * 10 + 5)
        } else {
            None
        };

        Ok(Self {
            device_model,
            device_color,

            left_battery,
            right_battery,
            case_battery,

            right_charging,
            left_charging,
            case_charging,
        })
    }
}
