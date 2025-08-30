use derivative::Derivative;
use getset::Getters;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Custom deserializer for key fields to cast numeric keys into strings
#[cfg(feature = "serde")]
pub(crate) fn deserialize_key_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: serde::Deserializer<'de>,
{
  use serde::de::Error;
  use serde_json::Value;

  let value = Value::deserialize(deserializer)?;
  match value {
    Value::String(s) => Ok(s),
    Value::Number(n) => Ok(n.to_string()),
    other => Err(Error::custom(format!("invalid type for key: {:?}", other))),
  }
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Project {
  id: Option<String>,
  name: Option<String>,
  description: Option<String>,

  layout: serde_json::Value,
  tracks: serde_json::Value,

  created_at: Option<u64>,
  updated_at: Option<u64>,
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
