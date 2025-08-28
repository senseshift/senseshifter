use derivative::Derivative;
use getset::Getters;

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "PascalCase")]
pub struct SdkEncryptedMessage {
  r#type: SdkEncryptedMessageType,
  key: Option<String>,
  data: Option<String>,
}

impl SdkEncryptedMessage {
  pub fn new(r#type: SdkEncryptedMessageType, key: Option<String>, data: Option<String>) -> Self {
    Self { r#type, key, data }
  }
}

#[derive(Derivative, strum::Display)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "PascalCase")]
pub enum SdkEncryptedMessageType {
  ServerKey,

  SdkClientKey,
  SdkData,
}

pub struct SdkData {
  r#type: SdkDataType,

  /// Value depends on the type
  ///
  /// From observations:
  /// - ServerReady: None
  /// - SdkPingAll: Some("")
  /// - SdkPlayDotMode: Some("<json>")
  /// - SdkPlayWithStartTime: Some("{\"eventName\": \"<event_name>\", <other_fields>}")
  /// - SdkStopByEventId: Some("<event_id>")
  data: Option<String>,
}

#[derive(Derivative, strum::Display)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "PascalCase")]
pub enum SdkDataType {
  ServerReady,
  ServerDevices,
  ServerEventNameList,
  ServerEventList,

  SdkPingAll,
  SdkPlayDotMode,
  SdkPlayWithStartTime,
  SdkPlayPathMode,
  SdkStopByEventId,
}