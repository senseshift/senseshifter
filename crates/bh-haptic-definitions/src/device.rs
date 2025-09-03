use derivative::Derivative;
use strum::{Display as StrumDisplay, EnumString};

#[derive(Derivative, StrumDisplay, EnumString)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DevicePosition {
  Head,
  Tactal,

  VestFront,
  VestBack,
  Vest,

  GloveL,
  GloveR,

  HandL,
  HandR,

  ForearmL,
  ForearmR,

  FootL,
  FootR,
}

impl DevicePosition {
  pub fn is_right(&self) -> bool {
    matches!(self, DevicePosition::GloveR | DevicePosition::HandR | DevicePosition::ForearmR | DevicePosition::FootR)
  }

  pub fn is_left(&self) -> bool {
    matches!(self, DevicePosition::GloveL | DevicePosition::HandL | DevicePosition::ForearmL | DevicePosition::FootL)
  }
}

#[derive(Derivative, StrumDisplay, EnumString)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeviceType {
  Tactosy2,
  TactosyH,
  TactosyF,

  TactVisor,
  TactFacial,

  Tactal,
  Tactosy,

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
