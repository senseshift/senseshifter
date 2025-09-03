use derivative::Derivative;
use getset::Getters;

use crate::EffectFeedbackPlaybackType;

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectDotMode {
  #[cfg_attr(feature = "serde", serde(default))]
  dot_connected: bool,

  feedback: Vec<EffectDotModeFeedback>
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectDotModeFeedback {
  start_time: u32,
  end_time: u32,
  playback_type: EffectFeedbackPlaybackType,
  point_list: Vec<EffectDotModePoint>,
}

/// todo: this might be probably also be a enum, if `index` is missing, in the `PATH_MODE` `x` and `y` fields are present in the same struct
/// See: [super::EffectPathModePoint]
#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectDotModePoint {
  /// reference to the `index` field of the [crate::LayoutPoint] in the [crate::Layout]
  index: u32,
  
  #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_handy::de::to_f64"))]
  intensity: f64,
}