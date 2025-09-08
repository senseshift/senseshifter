use crate::path_point_mapper::Mapper;
use crate::{
  DevicePosition, EffectDotMode, EffectMode, EffectPathMode, EffectPathModeFeedback,
  EffectPathModeMovingPattern, EffectPathModePoint, HapticEffect, Layout, TactFileProject, Track,
};
use approx::abs_diff_eq;
use std::collections::HashMap;
use std::sync::Arc;

impl TactFileProject {
  pub(crate) fn normalize_offset_angle_x(offset_angle_x: f64) -> f64 {
    let mut angle = offset_angle_x;
    if angle < 0.0 {
      angle += 360.0;
    }
    angle % 360.0
  }

  pub(crate) fn in_none_offset_angle_x(offset_angle_x: f64) -> bool {
    abs_diff_eq!(Self::normalize_offset_angle_x(offset_angle_x), 0.0)
  }

  pub(crate) fn in_none_offset_y(offset_y: f64) -> bool {
    abs_diff_eq!(offset_y, 0.0)
  }

  pub fn is_none_rotation(offset_angle_x: f64, offset_y: f64) -> bool {
    Self::in_none_offset_angle_x(offset_angle_x) && Self::in_none_offset_y(offset_y)
  }

  pub fn is_rotation_applicable(position: &DevicePosition) -> bool {
    matches!(
      position,
      DevicePosition::VestFront | DevicePosition::VestBack | DevicePosition::Head
    )
  }

  fn normalize_degrees(angle: f64) -> f64 {
    let result = angle % 360.0;
    if result < 0.0 { result + 360.0 } else { result }
  }

  fn ring_x_to_degrees(x: f64, rotation_deg: f64) -> f64 {
    Self::normalize_degrees(x * 360.0 + rotation_deg)
  }

  fn map_angle_to_device(
    angle_deg: f64,
    original_device: DevicePosition,
  ) -> anyhow::Result<(DevicePosition, f64)> {
    let a = Self::normalize_degrees(angle_deg);

    match original_device {
      DevicePosition::Head => Ok((DevicePosition::Head, a / 360.0)),
      DevicePosition::VestFront | DevicePosition::VestBack => {
        // For vest devices, map 0-180° to Front, 180-360° to Back
        if a < 180.0 {
          Ok((DevicePosition::VestFront, a / 180.0))
        } else {
          Ok((DevicePosition::VestBack, (a - 180.0) / 180.0))
        }
      }
      _ => anyhow::bail!(
        "Rotation not supported for device position: {:?}",
        original_device
      ),
    }
  }

  fn apply_rotation_to_path_point(
    x: f64,
    y: f64,
    intensity: f64,
    time: u32,
    original_device: DevicePosition,
    offset_angle_x: f64,
    offset_y: f64,
  ) -> Option<(DevicePosition, EffectPathModePoint)> {
    // Apply offset_y
    let y_prime = y - offset_y;
    if !(0.0..=1.0).contains(&y_prime) {
      return None;
    }

    // Apply rotation
    let angle_deg = Self::ring_x_to_degrees(x, offset_angle_x);
    let (target_device, local_x) = match Self::map_angle_to_device(angle_deg, original_device) {
      Ok(result) => result,
      Err(_) => return None, // Skip points for unsupported devices
    };

    // Clamp coordinates
    let clamped_x = local_x.clamp(0.0, 1.0);
    let clamped_y = y_prime.clamp(0.0, 1.0);
    let clamped_intensity = intensity.clamp(0.0, 1.0);

    let rotated_point = EffectPathModePoint::new(clamped_intensity, time, clamped_x, clamped_y);

    Some((target_device, rotated_point))
  }

  fn convert_dot_mode_to_path_mode(
    dot_mode: &EffectDotMode,
    device_position: DevicePosition,
    layout: &Layout,
  ) -> EffectPathMode {
    let layout_points = layout
      .layouts()
      .as_ref()
      .expect("Layout.layouts is None")
      .get(&device_position)
      .expect("Device position not found in layout");

    if *dot_mode.dot_connected() {
      // Create one large EffectPathModeFeedback with all points
      let mut all_points = Vec::new();

      for feedback in dot_mode.feedback() {
        for dot_point in feedback.point_list() {
          let layout_point = &layout_points[*dot_point.index() as usize];
          let path_point = EffectPathModePoint::new(
            *dot_point.intensity(),
            *feedback.start_time(),
            *layout_point.x(),
            *layout_point.y(),
          );
          all_points.push(path_point);
        }
      }

      let path_feedback = EffectPathModeFeedback::new(
        dot_mode.feedback()[0].playback_type().clone(), // Use first feedback's playback type
        EffectPathModeMovingPattern::ConstSpeed,
        true, // visible
        all_points,
      );

      EffectPathMode::new(vec![path_feedback])
    } else {
      // Create separate EffectPathModeFeedback for each original feedback
      let mut path_feedbacks = Vec::new();

      for feedback in dot_mode.feedback() {
        let mut points = Vec::new();

        for dot_point in feedback.point_list() {
          let layout_point = &layout_points[*dot_point.index() as usize];
          let path_point = EffectPathModePoint::new(
            *dot_point.intensity(),
            *feedback.start_time(),
            *layout_point.x(),
            *layout_point.y(),
          );
          points.push(path_point);
        }

        let path_feedback = EffectPathModeFeedback::new(
          feedback.playback_type().clone(),
          EffectPathModeMovingPattern::ConstSpeed,
          true, // visible
          points,
        );
        path_feedbacks.push(path_feedback);
      }

      EffectPathMode::new(path_feedbacks)
    }
  }

  pub fn apply_rotation(
    &self,
    _mapper: Arc<Box<dyn Mapper>>,
    offset_angle_x: f64,
    offset_y: f64,
  ) -> Self {
    if Self::is_none_rotation(offset_angle_x, offset_y) {
      return self.clone();
    }

    let mut new_project = self.clone();
    let mut new_tracks = Vec::new();

    for track in &new_project.tracks {
      let mut new_effects = Vec::new();

      for effect in track.effects() {
        // Convert all modes to PathMode first, then apply rotation
        let mut rotated_modes: HashMap<DevicePosition, EffectMode> = HashMap::new();

        for (device_position, mode) in effect.modes() {
          // Check if rotation is applicable for this device position
          if !Self::is_rotation_applicable(device_position) {
            // Skip rotation, keep the effect as is
            rotated_modes.insert(*device_position, mode.clone());
            continue;
          }

          let path_mode = match mode {
            EffectMode::DotMode { dot_mode } => {
              Self::convert_dot_mode_to_path_mode(dot_mode, *device_position, &new_project.layout)
            }
            EffectMode::PathMode { path_mode } => path_mode.clone(),
          };

          // Apply rotation to all path points and group by target device
          let mut device_groups: HashMap<DevicePosition, Vec<EffectPathModePoint>> = HashMap::new();

          for feedback in path_mode.feedback() {
            for point in feedback.point_list() {
              if let Some((target_device, rotated_point)) = Self::apply_rotation_to_path_point(
                *point.x(),
                *point.y(),
                *point.intensity(),
                *point.time(),
                *device_position,
                offset_angle_x,
                offset_y,
              ) {
                device_groups
                  .entry(target_device)
                  .or_default()
                  .push(rotated_point);
              }
            }
          }

          // Create new EffectPathMode for each target device
          for (target_device, points) in device_groups {
            if !points.is_empty() {
              let new_feedback = EffectPathModeFeedback::new(
                path_mode.feedback()[0].playback_type().clone(), // Use first feedback's playback type
                EffectPathModeMovingPattern::ConstSpeed,
                true, // visible
                points,
              );
              let new_path_mode = EffectPathMode::new(vec![new_feedback]);
              rotated_modes.insert(target_device, EffectMode::path_mode(new_path_mode));
            }
          }
        }

        if !rotated_modes.is_empty() {
          let new_effect = HapticEffect::new(
            effect.name().clone(),
            *effect.offset_time(),
            *effect.start_time(),
            rotated_modes,
          );
          new_effects.push(new_effect);
        }
      }

      let new_track = Track::new(*track.enable(), new_effects);
      new_tracks.push(new_track);
    }

    new_project.tracks = new_tracks;
    new_project
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{EffectFeedbackPlaybackType, LayoutPoint};

  #[test]
  fn test_normalize_offset_angle_x() {
    assert_eq!(TactFileProject::normalize_offset_angle_x(0.0), 0.0);
    assert_eq!(TactFileProject::normalize_offset_angle_x(360.0), 0.0);
    assert_eq!(TactFileProject::normalize_offset_angle_x(-360.0), 0.0);
    assert_eq!(TactFileProject::normalize_offset_angle_x(361.0), 1.0);
    assert_eq!(TactFileProject::normalize_offset_angle_x(-361.0), 359.0);
  }

  #[test]
  fn test_is_none_rotation() {
    assert!(TactFileProject::is_none_rotation(0.0, 0.0));
    assert!(TactFileProject::is_none_rotation(360.0, 0.0));
    assert!(TactFileProject::is_none_rotation(-360.0, 0.0));

    assert!(!TactFileProject::is_none_rotation(0.0, 1.0));
    assert!(!TactFileProject::is_none_rotation(10.0, 0.0));
    assert!(!TactFileProject::is_none_rotation(360.0, 1.0));
    assert!(!TactFileProject::is_none_rotation(-360.0, 1.0));
    assert!(!TactFileProject::is_none_rotation(0.0, -1.0));
  }

  fn create_test_layout() -> Layout {
    Layout {
      name: "Test Layout".to_string(),
      r#type: "Test Type".to_string(),
      layouts: Some(HashMap::from([
        (
          DevicePosition::VestFront,
          vec![
            LayoutPoint {
              index: 0,
              x: 0.0,
              y: 0.5,
            },
            LayoutPoint {
              index: 1,
              x: 0.5,
              y: 0.5,
            },
          ],
        ),
        (
          DevicePosition::Head,
          vec![LayoutPoint {
            index: 0,
            x: 0.25,
            y: 0.5,
          }],
        ),
      ])),
    }
  }

  #[test]
  fn test_no_rotation_returns_identical_project() {
    let layout = create_test_layout();
    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![Track::new(
        Some(true),
        vec![HapticEffect::new(
          Some("test".to_string()),
          1000,
          0,
          HashMap::from([(
            DevicePosition::VestFront,
            EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
              EffectFeedbackPlaybackType::None,
              EffectPathModeMovingPattern::ConstSpeed,
              true,
              vec![EffectPathModePoint::new(0.5, 500, 0.2, 0.6)],
            )])),
          )]),
        )],
      )],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let rotated = project.apply_rotation(
      Arc::new(Box::new(
        crate::path_point_mapper::InterpolatingMapperEvenGrid::new(8, 4),
      )),
      0.0,
      0.0,
    );

    // With no rotation, project should be identical
    assert_eq!(project.tracks.len(), rotated.tracks.len());
    assert_eq!(
      project.tracks[0].effects().len(),
      rotated.tracks[0].effects().len()
    );
  }

  #[test]
  fn test_non_rotatable_devices_remain_unchanged() {
    let layout = create_test_layout();
    let original_hand_effect =
      EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
        EffectFeedbackPlaybackType::None,
        EffectPathModeMovingPattern::ConstSpeed,
        true,
        vec![EffectPathModePoint::new(0.8, 100, 0.3, 0.7)],
      )]));

    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![Track::new(
        Some(true),
        vec![HapticEffect::new(
          Some("hand_test".to_string()),
          1000,
          0,
          HashMap::from([
            (DevicePosition::HandL, original_hand_effect.clone()),
            (DevicePosition::FootR, original_hand_effect.clone()),
            (DevicePosition::GloveL, original_hand_effect.clone()),
          ]),
        )],
      )],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let rotated = project.apply_rotation(
      Arc::new(Box::new(
        crate::path_point_mapper::InterpolatingMapperEvenGrid::new(8, 4),
      )),
      90.0,
      0.1,
    );

    // Non-rotatable devices should remain exactly the same
    let rotated_effect = &rotated.tracks[0].effects()[0];

    assert!(rotated_effect.modes().contains_key(&DevicePosition::HandL));
    assert!(rotated_effect.modes().contains_key(&DevicePosition::FootR));
    assert!(rotated_effect.modes().contains_key(&DevicePosition::GloveL));

    // Verify the effects are identical to original
    assert_eq!(
      rotated_effect.modes()[&DevicePosition::HandL],
      original_hand_effect
    );
    assert_eq!(
      rotated_effect.modes()[&DevicePosition::FootR],
      original_hand_effect
    );
    assert_eq!(
      rotated_effect.modes()[&DevicePosition::GloveL],
      original_hand_effect
    );
  }

  #[test]
  fn test_rotatable_devices_are_transformed() {
    let layout = create_test_layout();
    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![Track::new(
        Some(true),
        vec![HapticEffect::new(
          Some("vest_test".to_string()),
          1000,
          0,
          HashMap::from([(
            DevicePosition::VestFront,
            EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
              EffectFeedbackPlaybackType::None,
              EffectPathModeMovingPattern::ConstSpeed,
              true,
              vec![EffectPathModePoint::new(0.8, 100, 0.0, 0.5)], // x=0.0 should stay on VestFront with no rotation
            )])),
          )]),
        )],
      )],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    // Apply 180-degree rotation - should move VestFront to VestBack
    let rotated = project.apply_rotation(
      Arc::new(Box::new(
        crate::path_point_mapper::InterpolatingMapperEvenGrid::new(8, 4),
      )),
      180.0,
      0.0,
    );

    let rotated_effect = &rotated.tracks[0].effects()[0];

    // Original VestFront effect should be gone
    assert!(
      !rotated_effect
        .modes()
        .contains_key(&DevicePosition::VestFront)
    );

    // Should now have VestBack effect
    assert!(
      rotated_effect
        .modes()
        .contains_key(&DevicePosition::VestBack)
    );
  }

  #[test]
  fn test_mixed_rotatable_and_non_rotatable_devices() {
    let layout = create_test_layout();
    let hand_effect =
      EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
        EffectFeedbackPlaybackType::None,
        EffectPathModeMovingPattern::ConstSpeed,
        true,
        vec![EffectPathModePoint::new(0.9, 200, 0.4, 0.6)],
      )]));

    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![Track::new(
        Some(true),
        vec![HapticEffect::new(
          Some("mixed_test".to_string()),
          1000,
          0,
          HashMap::from([
            (
              DevicePosition::VestFront,
              EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
                EffectFeedbackPlaybackType::None,
                EffectPathModeMovingPattern::ConstSpeed,
                true,
                vec![EffectPathModePoint::new(0.7, 150, 0.0, 0.5)], // x=0.0 with 180° rotation should move to VestBack
              )])),
            ),
            (DevicePosition::HandL, hand_effect.clone()),
            (DevicePosition::FootR, hand_effect.clone()),
          ]),
        )],
      )],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let rotated = project.apply_rotation(
      Arc::new(Box::new(
        crate::path_point_mapper::InterpolatingMapperEvenGrid::new(8, 4),
      )),
      180.0,
      0.0,
    );

    let rotated_effect = &rotated.tracks[0].effects()[0];

    // Non-rotatable devices should remain unchanged
    assert_eq!(rotated_effect.modes()[&DevicePosition::HandL], hand_effect);
    assert_eq!(rotated_effect.modes()[&DevicePosition::FootR], hand_effect);

    // Rotatable device should be transformed
    assert!(
      !rotated_effect
        .modes()
        .contains_key(&DevicePosition::VestFront)
    );
    assert!(
      rotated_effect
        .modes()
        .contains_key(&DevicePosition::VestBack)
    );
  }
}
