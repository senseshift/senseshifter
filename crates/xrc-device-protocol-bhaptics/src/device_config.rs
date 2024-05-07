use derivative::Derivative;
use getset::Getters;
use serde::{Deserialize, Serialize};

static DEVICE_CONFIGURATION_JSON: &str = include_str!("device-config.json");

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Position {
  Vest,

  Head,

  ForearmR,
  ForearmL,

  HandR,
  HandL,

  GloveR,
  GloveL,

  FootR,
  FootL,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BHapticsDeviceType {
  TactBelt,
  Tactosy2,
  TactosyH,
  TactosyF,
  TactVisor,
  Tactal,
  Tactosy,
  Tactot,
  Tactot2,
  Tactot3,
  TactSuitX40,
  TactSuitX16,
  TactGloveL,
  TactGloveR,
  TactFacial,
  TactThermoHL,
  TactThermoHR,
  TactThermoFL,
  TactThermoFR,
}

impl From<BHapticsDeviceType> for String {
  fn from(val: BHapticsDeviceType) -> Self {
    serde_json::to_string(&val).expect("Always valid JSON string")
  }
}

impl BHapticsDeviceType {
  pub(crate) fn product_name(&self) -> String {
    match self {
      BHapticsDeviceType::TactBelt => "TactBelt".to_string(),
      BHapticsDeviceType::Tactosy | BHapticsDeviceType::Tactosy2 => "Tactosy2".to_string(),
      BHapticsDeviceType::TactosyH
      | BHapticsDeviceType::TactThermoHL
      | BHapticsDeviceType::TactThermoHR => "TactosyH".to_string(),
      BHapticsDeviceType::TactosyF
      | BHapticsDeviceType::TactThermoFL
      | BHapticsDeviceType::TactThermoFR => "TactosyF".to_string(),
      BHapticsDeviceType::TactVisor => "TactVisor".to_string(),
      BHapticsDeviceType::Tactal | BHapticsDeviceType::TactFacial => "Tactal".to_string(),
      BHapticsDeviceType::Tactot | BHapticsDeviceType::Tactot2 | BHapticsDeviceType::Tactot3 => {
        "Tactot".to_string()
      }
      BHapticsDeviceType::TactSuitX40 => "TactSuitX40".to_string(),
      BHapticsDeviceType::TactSuitX16 => "TactSuitX16".to_string(),
      BHapticsDeviceType::TactGloveL | BHapticsDeviceType::TactGloveR => "TactGlove".to_string(),
    }
  }
}

#[derive(Derivative, Getters, Deserialize, Clone, PartialEq, Eq, Hash)]
#[derivative(Debug)]
pub struct BHapticsDeviceIdentifier {
  #[get = "pub"]
  #[serde(rename = "name")]
  device: BHapticsDeviceType,

  #[get = "pub"]
  #[serde(rename = "nameContains")]
  #[derivative(Debug = "ignore")]
  name_contains: String,

  #[get = "pub"]
  #[serde(rename = "rawValue")]
  #[derivative(Debug = "ignore")]
  appearance: u16,

  #[get = "pub"]
  #[serde(rename = "candidatePositions")]
  #[derivative(Debug = "ignore")]
  candidate_positions: Vec<Position>,
}

pub(crate) fn load_device_identifiers() -> Vec<BHapticsDeviceIdentifier> {
  serde_json::from_str(DEVICE_CONFIGURATION_JSON)
    .expect("Failed to parse device configuration JSON")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_load_device_identifiers() {
    let _identifiers = load_device_identifiers();
  }
}
