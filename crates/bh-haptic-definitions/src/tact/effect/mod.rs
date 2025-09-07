mod dot;
mod path;

pub use dot::*;
pub use path::*;

use crate::DevicePosition;
use crate::traits::ScaleEffect;
use derivative::Derivative;
use getset::Getters;
use std::collections::HashMap;

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HapticEffect {
  name: Option<String>,

  #[cfg_attr(feature = "serde", serde(default))]
  offset_time: u32,

  #[cfg_attr(feature = "serde", serde(default))]
  start_time: u32,

  modes: HashMap<DevicePosition, EffectMode>,
}

impl HapticEffect {
  pub fn new(
    name: Option<String>,
    offset_time: u32,
    start_time: u32,
    modes: HashMap<DevicePosition, EffectMode>,
  ) -> Self {
    Self {
      name,
      offset_time,
      start_time,
      modes,
    }
  }
}

impl ScaleEffect for HapticEffect {
  #[inline]
  fn scale_effect(&mut self, duration: f64, intensity: f64) {
    self.offset_time = ((self.offset_time as f64) * duration) as u32;
    self.start_time = ((self.start_time as f64) * duration) as u32;
    self
      .modes
      .values_mut()
      .for_each(|m| m.scale_effect(duration, intensity));
  }
}

/// From the clients I always receive both `dotMode` and `pathMode` fields, but from observation,
/// only the one, selected by the `mode` JSON field is used, so I assume we might optimize their
/// struct to enum.
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "mode", rename_all = "camelCase"))]
pub enum EffectMode {
  #[cfg_attr(
    feature = "serde",
    serde(rename = "DOT_MODE", rename_all = "camelCase")
  )]
  DotMode { dot_mode: EffectDotMode },
  #[cfg_attr(
    feature = "serde",
    serde(rename = "PATH_MODE", rename_all = "camelCase")
  )]
  PathMode { path_mode: EffectPathMode },
}

impl EffectMode {
  pub fn dot_mode(dot_mode: EffectDotMode) -> Self {
    Self::DotMode { dot_mode }
  }

  pub fn path_mode(path_mode: EffectPathMode) -> Self {
    Self::PathMode { path_mode }
  }
}

impl ScaleEffect for EffectMode {
  #[inline]
  fn scale_effect(&mut self, duration: f64, intensity: f64) {
    match self {
      EffectMode::DotMode { dot_mode } => {
        dot_mode.scale_effect(duration, intensity);
      }
      EffectMode::PathMode { path_mode } => {
        path_mode.scale_effect(duration, intensity);
      }
    }
  }
}

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EffectFeedbackPlaybackType {
  /// No interpolation
  #[cfg_attr(feature = "serde", serde(rename = "NONE"))]
  None,

  /// Linear fade in
  #[cfg_attr(feature = "serde", serde(rename = "FADE_IN"))]
  FadeIn,

  /// Linear fade out.
  #[cfg_attr(feature = "serde", serde(rename = "FADE_OUT"))]
  FadeOut,

  /// Linear fade in and out.
  #[cfg_attr(feature = "serde", serde(rename = "FADE_IN_OUT"))]
  FadeInOut,
}

impl EffectFeedbackPlaybackType {
  pub fn apply(&self, ratio: f64, intensity: f64) -> f64 {
    match self {
      EffectFeedbackPlaybackType::None => intensity,
      EffectFeedbackPlaybackType::FadeIn => intensity * ratio,
      EffectFeedbackPlaybackType::FadeOut => intensity * (1.0 - ratio),
      EffectFeedbackPlaybackType::FadeInOut => intensity * (1.0 - (2.0 * ratio - 1.0).abs()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_effect_playback_type_apply() {
    let none = EffectFeedbackPlaybackType::None;

    assert_eq!(none.apply(0.0, 1.0), 1.0);
    assert_eq!(none.apply(0.5, 1.0), 1.0);
    assert_eq!(none.apply(1.0, 1.0), 1.0);

    let fade_in = EffectFeedbackPlaybackType::FadeIn;

    assert_eq!(fade_in.apply(0.0, 1.0), 0.0);
    assert_eq!(fade_in.apply(0.5, 1.0), 0.5);
    assert_eq!(fade_in.apply(1.0, 1.0), 1.0);

    let fade_out = EffectFeedbackPlaybackType::FadeOut;
    assert_eq!(fade_out.apply(0.0, 1.0), 1.0);
    assert_eq!(fade_out.apply(0.5, 1.0), 0.5);
    assert_eq!(fade_out.apply(1.0, 1.0), 0.0);

    let fade_in_out = EffectFeedbackPlaybackType::FadeInOut;
    assert_eq!(fade_in_out.apply(0.0, 1.0), 0.0);
    assert_eq!(fade_in_out.apply(0.5, 1.0), 1.0);
    assert_eq!(fade_in_out.apply(1.0, 1.0), 0.0);
  }
}
