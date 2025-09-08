use crate::EffectFeedbackPlaybackType;
use crate::traits::ScaleEffect;
use derivative::Derivative;
use getset::Getters;

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
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

impl ScaleEffect for EffectPathMode {
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

impl ScaleEffect for EffectPathModeFeedback {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
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

impl ScaleEffect for EffectPathModePoint {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    self.intensity *= intensity;
    self.time = ((self.time as f64) * duration_scale) as u32;
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_effect_path_mode_scale_effect() {
    let mut effect = EffectPathMode {
      feedback: vec![EffectPathModeFeedback {
        moving_pattern: EffectPathModeMovingPattern::ConstSpeed,
        playback_type: EffectFeedbackPlaybackType::None,
        visible: true,
        point_list: vec![
          EffectPathModePoint {
            intensity: 1.0,
            time: 2000,
            x: 0.1,
            y: 0.2,
          },
          EffectPathModePoint {
            intensity: 0.8,
            time: 3000,
            x: 0.4,
            y: 0.5,
          },
        ],
      }],
    };

    effect.scale_effect(1.2, 0.5);

    assert_eq!(effect.feedback.len(), 1);
    assert_eq!(effect.feedback[0].point_list.len(), 2);

    assert_eq!(effect.feedback[0].point_list[0].intensity, 0.5);
    assert_eq!(effect.feedback[0].point_list[0].time, 2400);
    assert_eq!(effect.feedback[0].point_list[0].x, 0.1);
    assert_eq!(effect.feedback[0].point_list[0].y, 0.2);

    assert_eq!(effect.feedback[0].point_list[1].intensity, 0.4);
    assert_eq!(effect.feedback[0].point_list[1].time, 3600);
    assert_eq!(effect.feedback[0].point_list[1].x, 0.4);
    assert_eq!(effect.feedback[0].point_list[1].y, 0.5);

    assert_eq!(
      effect.feedback[0].moving_pattern,
      EffectPathModeMovingPattern::ConstSpeed
    );
    assert_eq!(
      effect.feedback[0].playback_type,
      EffectFeedbackPlaybackType::None
    );
    assert!(effect.feedback[0].visible);
  }
}
