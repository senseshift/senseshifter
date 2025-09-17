use derivative::Derivative;
use strum::{Display as StrumDisplay, EnumString, FromRepr};

#[derive(Derivative, StrumDisplay, EnumString, FromRepr)]
#[derivative(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum DevicePosition {
  Vest = 0,

  ForearmL = 1,
  ForearmR = 2,

  Head = 3,

  HandL = 4,
  HandR = 5,

  FootL = 6,
  FootR = 7,

  GloveL = 8,
  GloveR = 9,

  Tactal = 200,

  VestFront = 201,
  VestBack = 202,
}

impl DevicePosition {
  pub fn is_right(&self) -> bool {
    matches!(
      self,
      DevicePosition::GloveR
        | DevicePosition::HandR
        | DevicePosition::ForearmR
        | DevicePosition::FootR
    )
  }

  pub fn is_left(&self) -> bool {
    matches!(
      self,
      DevicePosition::GloveL
        | DevicePosition::HandL
        | DevicePosition::ForearmL
        | DevicePosition::FootL
    )
  }

  #[cfg(feature = "serde")]
  pub fn serialize_as_repr<S>(position: &DevicePosition, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_u8(*position as u8)
  }

  #[cfg(feature = "serde")]
  pub fn deserialize_from_repr<'de, D>(deserializer: D) -> Result<DevicePosition, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    use serde::de::{Deserialize, Error, Unexpected};

    let repr_value = u8::deserialize(deserializer)?;
    DevicePosition::from_repr(repr_value).ok_or_else(|| {
      Error::invalid_value(
        Unexpected::Unsigned(repr_value as u64),
        &"valid device position",
      )
    })
  }
}

#[derive(Derivative, StrumDisplay, EnumString)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeviceType {}

#[derive(Derivative, StrumDisplay, EnumString)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeviceModel {
  Tactosy,
  Tactosy2,

  /// Tactosy for Hands
  TactosyH,

  /// Tactosy for Feet
  TactosyF,

  /// Developer Kit of Haptic Vests with 40 motors
  Tactot,
  /// Developer Kit of Haptic Vests with 40 motors
  Tactot2,
  /// Developer Kit of Haptic Vests with 40 motors
  Tactot3,

  /// 1st generation of the Vest with 16 motors
  TactSuitX16,
  /// 1st generation of the Vest with 40 motors
  TactSuitX40,
  /// 2nd generation of the Vest with 16 motors
  TactSuitAir,
  /// 2nd generation of the Vest with 32 motors
  TactSuitPro,

  /// 1st generation of Facial Interface with 5 motors
  Tactal,
  /// 2nd generation of Facial Interface with 4 motors
  TactVisor,
  // GloveL,
  // GloveR,
  // GloveDK3L,
  // GloveDK3R,
}

impl DeviceModel {
  pub fn possible_positions(&self) -> Vec<DevicePosition> {
    match self {
      DeviceModel::Tactosy | DeviceModel::Tactosy2 => {
        vec![DevicePosition::ForearmL, DevicePosition::ForearmR]
      }
      DeviceModel::TactosyH => vec![DevicePosition::HandL, DevicePosition::HandR],
      DeviceModel::TactosyF => vec![DevicePosition::FootL, DevicePosition::FootR],
      DeviceModel::Tactot
      | DeviceModel::Tactot2
      | DeviceModel::Tactot3
      | DeviceModel::TactSuitX16
      | DeviceModel::TactSuitX40
      | DeviceModel::TactSuitAir
      | DeviceModel::TactSuitPro => {
        vec![DevicePosition::Vest]
      }
      DeviceModel::Tactal | DeviceModel::TactVisor => {
        vec![DevicePosition::Head]
      }
    }
  }
}
