use std::collections::HashMap;
use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
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
#[derivative(Debug, Clone, PartialEq, Eq)]
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
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionTactFilePattern {
  position: Option<String>,
  tact_file: HapticDefinitionTactFile,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionTactFile {
  name: Option<String>,

  #[serde(default)]
  tracks: Vec<HapticDefinitionTrack>,

  layout: TactFileLayout,
}

/// Really looks like a `track` field from [bh_sdk::v2::Project]
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionTrack {
  enable: Option<bool>,

  #[serde(default)]
  effects: Vec<HapticDefinitionEffect>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionEffect {
  name: Option<String>,
  offset_time: Option<u32>,
  start_time: Option<u32>,

  /// todo: map key is enum probably, same as `layouts` field in the [TactFileLayout]
  modes: HashMap<String, HapticDefinitionEffectMode>,
}

/// From the clients I always receive both `dotMode` and `pathMode` fields, but from observation,
/// only the one, selected by the `mode` JSON field is used, so I assume we might optimize their
/// struct to enum.
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum HapticDefinitionEffectMode {
  #[serde(rename = "DOT_MODE", rename_all = "camelCase")]
  DotMode {
    dot_mode: HapticDefinitionEffectDotMode,
  },
  #[serde(rename = "PATH_MODE", rename_all = "camelCase")]
  PathMode {
    path_mode: HapticDefinitionEffectPathMode,
  },
}

// #[derive(Derivative, Serialize, Deserialize)]
// #[derivative(Debug, Clone, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct HapticDefinitionEffectMode {
//   dot_mode: HapticDefinitionEffectDotMode,
//   path_mode: HapticDefinitionEffectPathMode,
//   mode: HapticDefinitionEffectModeEnum,
// }
//
// #[derive(Derivative, Serialize, Deserialize)]
// #[derivative(Debug, Clone, PartialEq, Eq)]
// pub enum HapticDefinitionEffectModeEnum {
//   #[serde(rename = "DOT_MODE")]
//   DotMode,
//   #[serde(rename = "PATH_MODE")]
//   PathMode,
// }

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionEffectDotMode {
  #[serde(default)]
  dot_connected: bool,

  feedback: Vec<HapticDefinitionEffectDotModeFeedback>
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionEffectDotModeFeedback {
  start_time: u32,
  end_time: u32,
  playback_type: HapticDefinitionEffectFeedbackPlaybackType,
  point_list: Vec<HapticDefinitionEffectDotModePoint>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
pub enum HapticDefinitionEffectFeedbackPlaybackType {
  #[serde(rename = "NONE")]
  None,
  #[serde(rename = "FADE_IN")]
  FadeIn,
  #[serde(rename = "FADE_OUT")]
  FadeOut,
  #[serde(rename = "FADE_IN_OUT")]
  FadeInOut,
}

/// todo: this might be probably also be a enum, if `index` is missing, in the `PATH_MODE` `x` and `y` fields are present in the same struct
/// See: [HapticDefinitionEffectPathModePoint]
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionEffectDotModePoint {
  /// reference to the `index` field of the [LayoutPoint] in the [TactFileLayout]
  index: u32,
  intensity: f64,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionEffectPathMode {
  feedback: Vec<HapticDefinitionEffectPathModeFeedback>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionEffectPathModeFeedback {
  playback_type: HapticDefinitionEffectFeedbackPlaybackType,
  moving_pattern: HapticDefinitionEffectPathModeMovingPattern,
  visible: bool,
  point_list: Vec<HapticDefinitionEffectPathModePoint>,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionEffectPathModePoint {
  intensity: f64,
  time: u32,
  x: f64,
  y: f64,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
pub enum HapticDefinitionEffectPathModeMovingPattern {
  #[serde(rename = "CONST_SPEED")]
  ConstSpeed,
  #[serde(rename = "CONST_TDM")]
  ConstTdm,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TactFileLayout {
  name: String,

  /// todo: probably enum
  r#type: String,

  /// In `v4` I've only seen `null` here so far, but I've implemented this based on the `v2`.
  ///
  /// todo: map key is also enum probably
  layouts: Option<HashMap<String, Vec<LayoutPoint>>>,
}


/// todo: rename struct
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutPoint {
  index: u32,
  x: f64,
  y: f64,
}
