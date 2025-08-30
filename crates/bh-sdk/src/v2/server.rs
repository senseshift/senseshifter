use derivative::Derivative;
use getset::Getters;

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "PascalCase"))]
pub enum ServerMessage {
  ConnectedDeviceCount(u32),
  ActiveKeys(Vec<String>),
  RegisteredKeys(Vec<String>),
  ConnectedPositions,
  ConnectionPositions,
  Status(ServerStatusMessage),
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "PascalCase"))]
pub struct ServerStatusMessage {
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
