use getset::Getters;
use serde::Deserialize;

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

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
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

impl BHapticsDeviceType {
  pub(crate) fn product_name(&self) -> String {
    return match self {
      BHapticsDeviceType::TactBelt => "TactBelt".to_string(),
      BHapticsDeviceType::Tactosy | BHapticsDeviceType::Tactosy2 => "Tactosy2".to_string(),
      BHapticsDeviceType::TactosyH | BHapticsDeviceType::TactThermoHL | BHapticsDeviceType::TactThermoHR => {
        "TactosyH".to_string()
      }
      BHapticsDeviceType::TactosyF | BHapticsDeviceType::TactThermoFL | BHapticsDeviceType::TactThermoFR => {
        "TactosyF".to_string()
      }
      BHapticsDeviceType::TactVisor => "TactVisor".to_string(),
      BHapticsDeviceType::Tactal | BHapticsDeviceType::TactFacial => "Tactal".to_string(),
      BHapticsDeviceType::Tactot | BHapticsDeviceType::Tactot2 | BHapticsDeviceType::Tactot3 => "Tactot".to_string(),
      BHapticsDeviceType::TactSuitX40 => "TactSuitX40".to_string(),
      BHapticsDeviceType::TactSuitX16 => "TactSuitX16".to_string(),
      BHapticsDeviceType::TactGloveL | BHapticsDeviceType::TactGloveR => "TactGlove".to_string(),
    };
  }
}

#[derive(Getters, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BHapticsDeviceIdentifier {
  #[get = "pub"]
  #[serde(rename = "name")]
  device: BHapticsDeviceType,

  #[get = "pub"]
  #[serde(rename = "nameContains")]
  name_contains: String,

  #[get = "pub"]
  #[serde(rename = "rawValue")]
  appearance: u16,

  #[get = "pub"]
  #[serde(rename = "candidatePositions")]
  candidate_positions: Vec<Position>,
}

pub(crate) fn load_device_identifiers() -> Vec<BHapticsDeviceIdentifier> {
  serde_json::from_str(DEVICE_CONFIGURATION_JSON).expect("Failed to parse device configuration JSON")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_load_device_identifiers() {
    let _identifiers = load_device_identifiers();
  }
}