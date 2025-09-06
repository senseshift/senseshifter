use derivative::Derivative;
use strum::{Display as StrumDisplay, EnumString, FromRepr};

#[derive(Derivative, StrumDisplay, EnumString, FromRepr)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
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
}

#[derive(Derivative, StrumDisplay, EnumString)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeviceType {
  Tactosy,
  Tactosy2,
  TactosyH,
  TactosyF,

  TactVisor,
  TactFacial,

  Tactal,

  Tactot,
  Tactot2,
  Tactot3,
  TactSuitX40,
  TactSuitX16,
  TactSuitPro,
  TactSuitAir,

  TactGloveL,
  TactGloveR,

  Hand,
  Foot,
}
