use crate::tact::{
  EffectDotMode, EffectDotModeFeedback, EffectDotModePoint, EffectMode, EffectPathMode,
  EffectPathModeFeedback, EffectPathModePoint, HapticEffect, TactFileProject, Track,
};

#[cfg(test)]
use crate::{DevicePosition, EffectFeedbackPlaybackType, EffectPathModeMovingPattern};
#[cfg(test)]
use std::collections::HashMap;

pub trait ScaleEffect {
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64);
}

impl ScaleEffect for Track {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    self
      .effects_mut()
      .iter_mut()
      .for_each(|e| e.scale_effect(duration_scale, intensity));
  }
}

impl ScaleEffect for TactFileProject {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    self
      .tracks_mut()
      .iter_mut()
      .for_each(|t| t.scale_effect(duration_scale, intensity));
  }
}

impl ScaleEffect for HapticEffect {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    *self.offset_time_mut() = ((*self.offset_time() as f64) * duration_scale) as u32;
    *self.start_time_mut() = ((*self.start_time() as f64) * duration_scale) as u32;
    self
      .modes_mut()
      .values_mut()
      .for_each(|m| m.scale_effect(duration_scale, intensity));
  }
}

impl ScaleEffect for EffectMode {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    match self {
      EffectMode::DotMode { dot_mode } => {
        dot_mode.scale_effect(duration_scale, intensity);
      }
      EffectMode::PathMode { path_mode } => {
        path_mode.scale_effect(duration_scale, intensity);
      }
    }
  }
}

impl ScaleEffect for EffectDotMode {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    self
      .feedback_mut()
      .iter_mut()
      .for_each(|f| f.scale_effect(duration_scale, intensity));
  }
}

impl ScaleEffect for EffectDotModeFeedback {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    *self.start_time_mut() = ((*self.start_time() as f64) * duration_scale) as u32;
    *self.end_time_mut() = ((*self.end_time() as f64) * duration_scale) as u32;
    self
      .point_list_mut()
      .iter_mut()
      .for_each(|p| p.scale_effect(duration_scale, intensity));
  }
}

impl ScaleEffect for EffectDotModePoint {
  fn scale_effect(&mut self, _duration_scale: f64, intensity: f64) {
    *self.intensity_mut() *= intensity;
  }
}

impl ScaleEffect for EffectPathMode {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    self
      .feedback_mut()
      .iter_mut()
      .for_each(|f| f.scale_effect(duration_scale, intensity));
  }
}

impl ScaleEffect for EffectPathModeFeedback {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    self
      .point_list_mut()
      .iter_mut()
      .for_each(|p| p.scale_effect(duration_scale, intensity));
  }
}

impl ScaleEffect for EffectPathModePoint {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    *self.intensity_mut() *= intensity;
    *self.time_mut() = ((*self.time() as f64) * duration_scale) as u32;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_haptic_effect_scale_effect() {
    let mut effect = HapticEffect::new(
      None,
      100,
      220,
      HashMap::from([(
        DevicePosition::VestFront,
        EffectMode::dot_mode(EffectDotMode::new(
          true,
          vec![EffectDotModeFeedback::new(
            0,
            1000,
            EffectFeedbackPlaybackType::None,
            vec![
              EffectDotModePoint::new(0, 1.0),
              EffectDotModePoint::new(1, 0.8),
            ],
          )],
        )),
      )]),
    );

    effect.scale_effect(1.2, 0.5);

    assert_eq!(*effect.offset_time(), 120); // 100 * 1.2
    assert_eq!(*effect.start_time(), 264); // 220 * 1.2
    assert_eq!(effect.modes().len(), 1);
  }

  #[test]
  fn test_track_scale_effect() {
    let mut track = Track::new(
      None,
      vec![HapticEffect::new(
        None,
        100,
        220,
        HashMap::from([(
          DevicePosition::VestFront,
          EffectMode::dot_mode(EffectDotMode::new(
            true,
            vec![EffectDotModeFeedback::new(
              0,
              1000,
              EffectFeedbackPlaybackType::None,
              vec![
                EffectDotModePoint::new(0, 1.0),
                EffectDotModePoint::new(1, 0.8),
              ],
            )],
          )),
        )]),
      )],
    );

    track.scale_effect(1.2, 0.5);

    assert_eq!(track.effects().len(), 1);
    assert_eq!(*track.effects()[0].offset_time(), 120); // 100 * 1.2
    assert_eq!(*track.effects()[0].start_time(), 264); // 220 * 1.2
  }

  #[test]
  fn test_effect_dot_mode_scale_effect() {
    let mut effect = EffectDotMode::new(
      true,
      vec![EffectDotModeFeedback::new(
        1000,
        2000,
        EffectFeedbackPlaybackType::None,
        vec![
          EffectDotModePoint::new(0, 1.0),
          EffectDotModePoint::new(1, 0.8),
        ],
      )],
    );

    effect.scale_effect(1.2, 0.5);

    assert!(*effect.dot_connected());

    assert_eq!(effect.feedback().len(), 1);

    assert_eq!(*effect.feedback()[0].start_time(), 1200);
    assert_eq!(*effect.feedback()[0].end_time(), 2400);

    assert_eq!(
      *effect.feedback()[0].playback_type(),
      EffectFeedbackPlaybackType::None
    );

    assert_eq!(effect.feedback()[0].point_list().len(), 2);

    assert_eq!(*effect.feedback()[0].point_list()[0].intensity(), 0.5);
    assert_eq!(*effect.feedback()[0].point_list()[1].intensity(), 0.4);
  }

  #[test]
  fn test_effect_path_mode_scale_effect() {
    let mut effect = EffectPathMode::new(vec![EffectPathModeFeedback::new(
      EffectFeedbackPlaybackType::None,
      EffectPathModeMovingPattern::ConstSpeed,
      true,
      vec![
        EffectPathModePoint::new(1.0, 2000, 0.1, 0.2),
        EffectPathModePoint::new(0.8, 3000, 0.4, 0.5),
      ],
    )]);

    effect.scale_effect(1.2, 0.5);

    assert_eq!(effect.feedback().len(), 1);
    assert_eq!(effect.feedback()[0].point_list().len(), 2);

    assert_eq!(*effect.feedback()[0].point_list()[0].intensity(), 0.5);
    assert_eq!(*effect.feedback()[0].point_list()[0].time(), 2400);
    assert_eq!(*effect.feedback()[0].point_list()[0].x(), 0.1);
    assert_eq!(*effect.feedback()[0].point_list()[0].y(), 0.2);

    assert_eq!(*effect.feedback()[0].point_list()[1].intensity(), 0.4);
    assert_eq!(*effect.feedback()[0].point_list()[1].time(), 3600);
    assert_eq!(*effect.feedback()[0].point_list()[1].x(), 0.4);
    assert_eq!(*effect.feedback()[0].point_list()[1].y(), 0.5);

    assert_eq!(
      *effect.feedback()[0].moving_pattern(),
      EffectPathModeMovingPattern::ConstSpeed
    );
    assert_eq!(
      *effect.feedback()[0].playback_type(),
      EffectFeedbackPlaybackType::None
    );
    assert!(*effect.feedback()[0].visible());
  }
}
