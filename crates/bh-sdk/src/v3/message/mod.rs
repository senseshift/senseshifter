use derivative::Derivative;
use getset::Getters;

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "message"))]
#[cfg_attr(feature = "serde", serde_with::serde_as)]
pub enum SdkMessage {
  SdkRequestAuth(
    #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))]
    SdkRequestAuthMessage,
  ),
  SdkPlayWithStartTime(
    #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))]
    SdkPlayWithStartTimeMessage,
  ),
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SdkRequestAuthMessage {
  cipher: String,
  application_id: String,
  nonce_hash_value: String,
  application_id_hash_value: String,
  sdk_api_key: String,
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SdkPlayWithStartTimeMessage {
  event_name: String,
  request_id: u32,
  start_millis: u64,
  intensity: f64,
  duration: f64,
  offset_angle_x: f64,
  offset_y: f64,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(feature = "serde")]
  #[test]
  fn test_encodes_message_as_string() {
    let msg = SdkMessage::SdkRequestAuth(SdkRequestAuthMessage {
      cipher: "cipher".to_string(),
      application_id: "app_id".to_string(),
      nonce_hash_value: "nonce".to_string(),
      application_id_hash_value: "app_hash".to_string(),
      sdk_api_key: "api_key".to_string(),
    });

    let json = serde_json::to_string(&msg).unwrap();

    let expected = r#"{"type":"SdkRequestAuth","message":"{\"cipher\":\"cipher\",\"applicationId\":\"app_id\",\"nonceHashValue\":\"nonce\",\"applicationIdHashValue\":\"app_hash\",\"sdkApiKey\":\"api_key\"}"}"#;
    assert_eq!(json, expected);
  }
}
