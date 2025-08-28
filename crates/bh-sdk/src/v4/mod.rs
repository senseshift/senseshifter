use derivative::Derivative;
use getset::Getters;

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "PascalCase")]
pub struct SdkEncryptedMessage {
  r#type: String,
  key: Option<String>,
  data: Option<String>,
}

impl SdkEncryptedMessage {
  pub fn new(r#type: String, key: Option<String>, data: Option<String>) -> Self {
    Self { r#type, key, data }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_deserialize_sdk_encrypted_message() {
    assert_eq!(
      serde_json::from_str::<SdkEncryptedMessage>(
        r#"{
            "Type":"SdkData",
            "Key":null,
            "Data":"base64="
          }"#
      )
      .unwrap(),
      SdkEncryptedMessage {
        r#type: "SdkData".to_string(),
        key: None,
        data: Some("base64=".to_string()),
      }
    );

    assert_eq!(
      serde_json::from_str::<SdkEncryptedMessage>(
        r#"{
            "Type":"ServerKey",
            "Key":"base64=",
            "Data":null
          }"#
      )
      .unwrap(),
      SdkEncryptedMessage {
        r#type: "ServerKey".to_string(),
        key: Some("base64=".to_string()),
        data: None,
      }
    );

    assert_eq!(
      serde_json::from_str::<SdkEncryptedMessage>(
        r#"{
            "Type":"SdkClientKey",
            "Key":"base64=",
            "Data":null
          }"#
      )
      .unwrap(),
      SdkEncryptedMessage {
        r#type: "SdkClientKey".to_string(),
        key: Some("base64=".to_string()),
        data: None,
      }
    );
  }
}
