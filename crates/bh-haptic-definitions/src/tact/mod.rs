mod effect;
mod frame;
mod rotation;
mod track;

pub use effect::*;
pub use frame::*;
pub use track::*;

use crate::DevicePosition;
use crate::traits::ScaleEffect;
use derivative::Derivative;
use derive_more::with_trait::Display;
use getset::Getters;
use std::collections::HashMap;

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HapticDefinitionTactFilePattern {
  position: String,
  tact_file: TactFileProject,
}

/// Schema for the `.tact` files
#[derive(Derivative, Getters, Display)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[display("{project:?}")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TactFile {
  project: TactFileProject,
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TactFileProject {
  #[cfg_attr(
    feature = "serde",
    serde(
      default,
      deserialize_with = "serde_handy::de::from_str_num_to_opt_string"
    )
  )]
  id: Option<String>,
  name: Option<String>,
  description: Option<String>,

  #[cfg_attr(feature = "serde", serde(default, alias = "Tracks"))]
  tracks: Vec<Track>,

  #[cfg_attr(feature = "serde", serde(alias = "Layout"))]
  layout: Layout,

  media_file_duration: Option<f64>,

  created_at: Option<u64>,
  updated_at: Option<u64>,
}

impl ScaleEffect for TactFileProject {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    self
      .tracks
      .iter_mut()
      .for_each(|t| t.scale_effect(duration_scale, intensity));
  }
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Layout {
  name: String,

  /// Presumably, a weird/old version [crate::DeviceModel] with additional options,
  /// such as `Hand`, `Foot`.
  /// But for the most part, what I've seen is the same.
  r#type: String,

  /// List of points to reference in tracks.
  layouts: Option<HashMap<DevicePosition, Vec<LayoutPoint>>>,
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LayoutPoint {
  index: u32,

  #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_handy::de::to_f64"))]
  x: f64,

  #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_handy::de::to_f64"))]
  y: f64,
}

#[cfg(test)]
mod tests {
  use super::Track;
  use super::*;

  #[test]
  fn test_tact_file_project_scale_effect() {
    let mut project = TactFileProject {
      id: Some("1".to_string()),
      name: Some("Test Project".to_string()),
      description: Some("A test project".to_string()),
      tracks: vec![Track::new(
        None,
        vec![HapticEffect::new(
          None,
          100,
          220,
          HashMap::from([
            (
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
            ),
            (
              DevicePosition::VestBack,
              EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
                EffectFeedbackPlaybackType::None,
                EffectPathModeMovingPattern::ConstSpeed,
                true,
                vec![
                  EffectPathModePoint::new(1.0, 0, 0.0, 0.0),
                  EffectPathModePoint::new(0.5, 500, 1.0, 1.0),
                  EffectPathModePoint::new(0.0, 1000, 0.0, 1.0),
                ],
              )])),
            ),
          ]),
        )],
      )],
      layout: Layout {
        name: "Test Layout".to_string(),
        r#type: "Test Type".to_string(),
        layouts: None,
      },
      media_file_duration: Some(10.0),
      created_at: Some(1625079600),
      updated_at: Some(1625079600),
    };

    project.scale_effect(1.2, 0.5);

    assert_eq!(project.tracks.len(), 1);

    assert_eq!(project.tracks[0].effects().len(), 1);
    assert_eq!(*project.tracks[0].effects()[0].offset_time(), 120); // 100 * 1.2
    assert_eq!(*project.tracks[0].effects()[0].start_time(), 264); // 220 * 0.5

    assert_eq!(project.tracks[0].effects()[0].modes().len(), 2);
  }
}
