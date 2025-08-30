use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionsMessage {
  id: Option<String>,
  create_time: Option<u64>,
  name: Option<String>,
  description: Option<String>,
  creator: Option<String>,
  workspace_id: Option<String>,
  version: Option<i64>,
  disable_validation: Option<bool>,
  category_options: Option<Vec<String>>,

  #[serde(default)]
  haptic_mappings: Vec<HapticDefinitionMapping>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionMapping {
  enable: Option<bool>,
  intensity: Option<u32>,
  key: String,
  category: Option<String>,
  description: Option<String>,
  update_time: Option<u64>,

  event_time: u32,

  #[serde(default)]
  tact_file_patterns: Vec<HapticDefinitionTactFilePattern>,

  /// todo: examine the structure of this field
  #[serde(default)]
  audio_file_patterns: Option<Vec<serde_json::Value>>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionTactFilePattern {
  position: Option<String>,
  tact_file: HapticDefinitionTactFile,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionTactFile {
  name: Option<String>,

  #[serde(default)]
  tracks: Vec<HapticDefinitionTrack>,
}

/// Really looks like a `track` field from [bh_sdk::v2::Project]
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionTrack {
  enable: Option<bool>,

  #[serde(default)]
  effects: Vec<HapticDefinitionEffect>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionEffect {
  name: Option<String>,
  offset_time: Option<u32>,
  start_time: Option<u32>,
}