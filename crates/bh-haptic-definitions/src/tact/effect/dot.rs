use derivative::Derivative;
use getset::{Getters, MutGetters};

use crate::EffectFeedbackPlaybackType;

#[derive(Derivative, Getters, MutGetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectDotMode {
  #[cfg_attr(feature = "serde", serde(default))]
  dot_connected: bool,

  feedback: Vec<EffectDotModeFeedback>,
}

impl EffectDotMode {
  pub fn new(dot_connected: bool, feedback: Vec<EffectDotModeFeedback>) -> Self {
    Self {
      dot_connected,
      feedback,
    }
  }
}

#[derive(Derivative, Getters, MutGetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectDotModeFeedback {
  start_time: u32,
  end_time: u32,
  playback_type: EffectFeedbackPlaybackType,
  point_list: Vec<EffectDotModePoint>,
}

impl EffectDotModeFeedback {
  pub fn new(
    start_time: u32,
    end_time: u32,
    playback_type: EffectFeedbackPlaybackType,
    point_list: Vec<EffectDotModePoint>,
  ) -> Self {
    Self {
      start_time,
      end_time,
      playback_type,
      point_list,
    }
  }
}

#[derive(Derivative, Getters, MutGetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffectDotModePoint {
  /// Reference to the `index` field of the [LayoutPoint](crate::LayoutPoint) in the [Layout](crate::Layout).
  /// At the same time is the motor index in the standard sense (see [InterpolatingMapperEvenGrid](crate::path_point_mapper::InterpolatingMapperEvenGrid) for index positions).
  ///
  /// Together they are used to apply rotation to the effect.
  index: u32,

  #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_handy::de::to_f64"))]
  intensity: f64,
}

impl EffectDotModePoint {
  pub fn new(index: u32, intensity: f64) -> Self {
    Self { index, intensity }
  }
}
