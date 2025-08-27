use derivative::Derivative;
use serde::Deserialize;

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "PascalCase")]
pub enum Message {
    Register(Vec<RegisterMessage>),
    Submit(Vec<SubmitMessage>)
}

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RegisterMessage {
    #[serde(rename = "Key", deserialize_with = "deserialize_key_to_string")]
    key: String,
    project: Project,
}

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Project {
    id: String,
    name: String,
    description: Option<String>,

    layout: serde_json::Value,
    tracks: serde_json::Value,

    created_at: Option<u64>,
    updated_at: Option<u64>,
}

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "PascalCase", tag = "Type")]
pub enum SubmitMessage {
    #[serde(rename = "turnOffAll")]
    TurnOffAll,
    #[serde(rename = "turnOff", rename_all = "PascalCase")]
    TurnOff {
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

// pub struct SubmitMessageFrame {
//     position: String,
// }

pub enum ResponseMessage {
    ConnectedDeviceCount(u32),
    ActiveKeys,
    RegisteredKeys,
    ConnectedPositions,
    ConnectionPositions,
    Status,
}

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HapticDotPoint {
    index: u32,
    intensity: u32,
}

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HapticPathPoint {
    x: f32,
    y: f32,
    intensity: u32,
    motor_count: u32,
}

// Custom deserializer for key fields to cast numeric keys into strings
fn deserialize_key_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        other => Err(D::Error::custom(format!("invalid type for key: {:?}", other))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_message() {
        assert_eq!(serde_json::from_str::<Message>(r#"{"Register": []}"#).unwrap(), Message::Register(vec![]));
        assert_eq!(serde_json::from_str::<Message>(r#"{"Submit": [{"Type": "turnOff", "Key": "example_key"}]}"#).unwrap(), Message::Submit(vec![SubmitMessage::TurnOff { key: "example_key".to_string() }]));
        assert_eq!(serde_json::from_str::<Message>(r#"{"Submit": [{"Type": "turnOffAll"}]}"#).unwrap(), Message::Submit(vec![SubmitMessage::TurnOffAll]));
        assert_eq!(serde_json::from_str::<Message>(r#"{"Submit": [{"Type": "frame", "Key": "example_key", "Frame": {"some": "data"}}]}"#).unwrap(), Message::Submit(vec![SubmitMessage::Frame { key: "example_key".to_string(), frame: serde_json::json!({"some": "data"}) }]));
    }
}
