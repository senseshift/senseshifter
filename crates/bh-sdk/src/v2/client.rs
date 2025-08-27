use derivative::Derivative;
use getset::Getters;

use super::Project;

#[cfg(feature = "serde")]
use super::deserialize_key_to_string;

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "PascalCase")]
pub struct ClientMessage {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  register: Option<Vec<ClientRegisterMessage>>,

  #[serde(default, skip_serializing_if = "Option::is_none")]
  submit: Option<Vec<ClientSubmitMessage>>,
}

impl ClientMessage {
  pub fn new(
    register: Option<Vec<ClientRegisterMessage>>,
    submit: Option<Vec<ClientSubmitMessage>>,
  ) -> Self {
    Self { register, submit }
  }

  pub fn new_register(messages: Vec<ClientRegisterMessage>) -> Self {
    Self::new(Some(messages), None)
  }

  pub fn new_submit(messages: Vec<ClientSubmitMessage>) -> Self {
    Self::new(None, Some(messages))
  }
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientRegisterMessage {
  #[serde(rename = "Key", deserialize_with = "deserialize_key_to_string")]
  key: String,
  project: Project,
}

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub enum ClientSubmitMessage {
  #[serde(rename = "turnOffAll")]
  TurnOffAll,

  #[serde(rename = "turnOff", rename_all = "PascalCase")]
  TurnOff {
    #[serde(rename = "Key", deserialize_with = "deserialize_key_to_string")]
    key: String,
  },

  #[serde(rename = "key", rename_all = "PascalCase")]
  Key {
    #[serde(rename = "Key", deserialize_with = "deserialize_key_to_string")]
    key: String,
  },

  #[serde(rename = "frame", rename_all = "PascalCase")]
  Frame {
    #[serde(rename = "Key", deserialize_with = "deserialize_key_to_string")]
    key: String,
    frame: serde_json::Value,
  },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_deserialize_message() {
    assert_eq!(
      serde_json::from_str::<ClientMessage>(r#"{"Register": []}"#).unwrap(),
      ClientMessage::new_register(vec![])
    );
    assert_eq!(
      serde_json::from_str::<ClientMessage>(
        r#"{"Submit": [{"Type": "turnOff", "Key": "example_key"}]}"#
      )
      .unwrap(),
      ClientMessage::new_submit(vec![ClientSubmitMessage::TurnOff {
        key: "example_key".to_string()
      }])
    );
    assert_eq!(
      serde_json::from_str::<ClientMessage>(r#"{"Submit": [{"Type": "turnOffAll"}]}"#).unwrap(),
      ClientMessage::new_submit(vec![ClientSubmitMessage::TurnOffAll])
    );
    assert_eq!(
      serde_json::from_str::<ClientMessage>(
        r#"{"Submit": [{"Type": "frame", "Key": "example_key", "Frame": {"some": "data"}}]}"#
      )
      .unwrap(),
      ClientMessage::new_submit(vec![ClientSubmitMessage::Frame {
        key: "example_key".to_string(),
        frame: serde_json::json!({"some": "data"})
      }])
    );
  }
}
