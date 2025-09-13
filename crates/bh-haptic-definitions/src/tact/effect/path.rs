use crate::EffectFeedbackPlaybackType;
use derivative::Derivative;
use getset::{Getters, MutGetters};

#[derive(Derivative, Getters, MutGetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectPathMode {
  feedback: Vec<EffectPathModeFeedback>,
}

impl EffectPathMode {
  pub fn new(feedback: Vec<EffectPathModeFeedback>) -> Self {
    Self { feedback }
  }
}

#[derive(Derivative, Getters, MutGetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectPathModeFeedback {
  playback_type: EffectFeedbackPlaybackType,
  moving_pattern: EffectPathModeMovingPattern,
  visible: bool,
  point_list: Vec<EffectPathModePoint>,
}

impl EffectPathModeFeedback {
  pub fn new(
    playback_type: EffectFeedbackPlaybackType,
    moving_pattern: EffectPathModeMovingPattern,
    visible: bool,
    point_list: Vec<EffectPathModePoint>,
  ) -> Self {
    Self {
      playback_type,
      moving_pattern,
      visible,
      point_list,
    }
  }
}

#[derive(Derivative, Getters, MutGetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectPathModePoint {
  #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_handy::de::to_f64"))]
  intensity: f64,

  time: u32,

  #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_handy::de::to_f64"))]
  x: f64,

  #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_handy::de::to_f64"))]
  y: f64,
}

impl EffectPathModePoint {
  pub fn new(intensity: f64, time: u32, x: f64, y: f64) -> Self {
    Self {
      intensity,
      time,
      x,
      y,
    }
  }
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
