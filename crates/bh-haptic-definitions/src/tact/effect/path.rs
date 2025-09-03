use derivative::Derivative;
use getset::Getters;
use crate::EffectFeedbackPlaybackType;

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectPathMode {
  feedback: Vec<EffectPathModeFeedback>,
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectPathModeFeedback {
  playback_type: EffectFeedbackPlaybackType,
  moving_pattern: EffectPathModeMovingPattern,
  visible: bool,
  point_list: Vec<EffectPathModePoint>,
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectPathModePoint {
  intensity: f64,
  time: u32,
  x: f64,
  y: f64,
}

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum EffectPathModeMovingPattern {
  #[cfg_attr(feature = "serde", serde(rename = "CONST_SPEED"))]
  ConstSpeed,
  #[cfg_attr(feature = "serde", serde(rename = "CONST_TDM"))]
  ConstTdm,
}