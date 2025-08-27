use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "PascalCase")]
pub enum ServerMessage {
  ConnectedDeviceCount(u32),
  ActiveKeys,
  RegisteredKeys(Vec<String>),
  ConnectedPositions,
  ConnectionPositions,
  Status,
}
