use derivative::Derivative;
use getset::Getters;

use bh_haptic_definitions::DevicePosition;

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "PascalCase"))]
pub struct ServerMessage {
  status: ServerStatus,
  active_keys: Vec<String>,
  registered_keys: Vec<String>,
  connected_positions: Vec<DevicePosition>,
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "PascalCase"))]
pub struct ServerStatus {
  #[cfg_attr(feature = "serde", serde(rename = "VestFront"))]
  vest_front: [u32; 20],

  #[cfg_attr(feature = "serde", serde(rename = "VestBack"))]
  vest_back: [u32; 20],

  #[cfg_attr(feature = "serde", serde(rename = "ForearmL"))]
  forearm_left: [u32; 6],

  #[cfg_attr(feature = "serde", serde(rename = "ForearmR"))]
  forearm_right: [u32; 6],

  #[cfg_attr(feature = "serde", serde(rename = "Head"))]
  head: [u32; 6],
}