use derivative::Derivative;
use getset::Getters;

use crate::EffectFeedbackPlaybackType;
use crate::traits::ScaleEffect;

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
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

impl ScaleEffect for EffectDotMode {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    self
      .feedback
      .iter_mut()
      .for_each(|f| f.scale_effect(duration_scale, intensity));
  }
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

impl ScaleEffect for EffectDotModeFeedback {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    self.start_time = ((self.start_time as f64) * duration_scale) as u32;
    self.end_time = ((self.end_time as f64) * duration_scale) as u32;
    self
      .point_list
      .iter_mut()
      .for_each(|p| p.scale_effect(duration_scale, intensity));
  }
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
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

impl ScaleEffect for EffectDotModePoint {
  fn scale_effect(&mut self, _duration_scale: f64, intensity: f64) {
    self.intensity *= intensity;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_effect_dot_mode_scale_effect() {
    let mut effect = EffectDotMode {
      dot_connected: true,
      feedback: vec![EffectDotModeFeedback {
        start_time: 1000,
        end_time: 2000,
        playback_type: EffectFeedbackPlaybackType::None,
        point_list: vec![
          EffectDotModePoint {
            index: 0,
            intensity: 1.0,
          },
          EffectDotModePoint {
            index: 1,
            intensity: 0.8,
          },
        ],
      }],
    };

    effect.scale_effect(1.2, 0.5);

    assert!(effect.dot_connected);

    assert_eq!(effect.feedback.len(), 1);

    assert_eq!(effect.feedback[0].start_time, 1200);
    assert_eq!(effect.feedback[0].end_time, 2400);

    assert_eq!(
      effect.feedback[0].playback_type,
      EffectFeedbackPlaybackType::None
    );

    assert_eq!(effect.feedback[0].point_list.len(), 2);

    assert_eq!(effect.feedback[0].point_list[0].intensity, 0.5);
    assert_eq!(effect.feedback[0].point_list[1].intensity, 0.4);
  }
}
