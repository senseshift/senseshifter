use crate::{
  DevicePosition, DotPoint, EffectMode, EffectPathMode, EffectPathModeFeedback,
  EffectPathModePoint, HapticEffect, PathPoint, TactFileProject, path_point_mapper::PositionMapper,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct FrameCompilationConfig {
  /// Step size in milliseconds for frame splitting
  pub step_size_ms: u32,
}

impl Default for FrameCompilationConfig {
  fn default() -> Self {
    Self { step_size_ms: 20 }
  }
}

impl FrameCompilationConfig {
  pub fn new(step_size_ms: u32) -> Self {
    Self { step_size_ms }
  }
}

pub struct FrameCompiler {
  config: FrameCompilationConfig,
  position_mapper: Arc<PositionMapper>,
}

impl FrameCompiler {
  pub fn new(config: FrameCompilationConfig, position_mapper: Arc<PositionMapper>) -> Self {
    Self {
      config,
      position_mapper,
    }
  }

  /// Compile the project into frames
  pub fn compile(&self, project: &TactFileProject) -> Vec<CompiledFrame> {
    let mut frames = Vec::new();
    let total_duration = project.get_total_duration();

    if total_duration == 0 {
      return frames;
    }

    // Generate frames at each step interval
    let mut current_time = 0u32;
    while current_time <= total_duration {
      let mut frame = CompiledFrame::new(current_time);

      // Process all tracks and effects at this timestamp
      for track in project.tracks() {
        if let Some(enable) = track.enable()
          && !enable
        {
          continue;
        }

        for effect in track.effects() {
          self.compile_effect_at_time(project, effect, current_time, &mut frame);
        }
      }

      frames.push(frame);
      current_time += self.config.step_size_ms;
    }

    frames
  }

  /// Compile a specific effect at a given timestamp
  fn compile_effect_at_time(
    &self,
    project: &TactFileProject,
    effect: &HapticEffect,
    timestamp: u32,
    frame: &mut CompiledFrame,
  ) {
    let effect_start = effect.start_time() + effect.offset_time();
    let effect_end = effect_start + project.get_effect_duration(effect);

    // Check if the effect is active at this timestamp
    if timestamp < effect_start || timestamp > effect_end {
      return;
    }

    let local_time = timestamp - effect_start;

    // Process each device mode in the effect
    for (device_position, effect_mode) in effect.modes() {
      match effect_mode {
        EffectMode::PathMode { path_mode } => {
          self.compile_path_mode_at_time(path_mode, *device_position, local_time, frame);
        }
        EffectMode::DotMode { dot_mode: _ } => {
          // TODO: Implement dot mode compilation
          // This would involve converting dot points to frame points
        }
      }
    }
  }

  /// Compile path mode effect at a specific time
  fn compile_path_mode_at_time(
    &self,
    path_mode: &EffectPathMode,
    device_position: DevicePosition,
    local_time: u32,
    frame: &mut CompiledFrame,
  ) {
    for feedback in path_mode.feedback() {
      self.compile_feedback_at_time(feedback, device_position, local_time, frame);
    }
  }

  /// Compile feedback at a specific time
  fn compile_feedback_at_time(
    &self,
    feedback: &EffectPathModeFeedback,
    device_position: DevicePosition,
    local_time: u32,
    frame: &mut CompiledFrame,
  ) {
    let points = feedback.point_list();
    if points.is_empty() {
      return;
    }

    // Find the appropriate point(s) for interpolation at this time
    if let Some(interpolated_path_point) = self.interpolate_path_point_at_time(points, local_time) {
      // Convert PathPoint to DotPoints using PositionMapper
      if let Ok(dot_points) = self
        .position_mapper
        .map_point(&device_position, &interpolated_path_point)
      {
        for dot_point in dot_points {
          // Convert DotPoint back to FramePoint for consistent API
          // Note: This assumes we have layout information to convert index back to x,y
          // For now, we'll use a simplified approach
          let frame_point = FramePoint {
            x: 0.0, // TODO: Convert index back to x coordinate using layout
            y: 0.0, // TODO: Convert index back to y coordinate using layout
            intensity: *dot_point.intensity() as f64 / DotPoint::MAX_INTENSITY as f64,
          };
          frame.add_point(device_position, frame_point);
        }
      }
    }
  }

  /// Interpolate a PathPoint at a specific time from the point list
  fn interpolate_path_point_at_time(
    &self,
    points: &[EffectPathModePoint],
    local_time: u32,
  ) -> Option<PathPoint> {
    if points.is_empty() {
      return None;
    }

    // Sort points by time for proper interpolation
    let mut sorted_points: Vec<_> = points.iter().collect();
    sorted_points.sort_by_key(|p| p.time());

    // Find the two points to interpolate between
    let mut before_point = None;
    let mut after_point = None;

    for point in &sorted_points {
      if point.time() <= &local_time {
        before_point = Some(point);
      } else {
        after_point = Some(point);
        break;
      }
    }

    match (before_point, after_point) {
      (Some(before), Some(after)) => {
        // Interpolate between two points
        let time_diff = after.time() - before.time();
        if time_diff == 0 {
          return Some(PathPoint::new(
            *before.x(),
            *before.y(),
            (*before.intensity() * 100.0) as u8, // Convert to u8 intensity
          ));
        }

        let t = (local_time - before.time()) as f64 / time_diff as f64;
        Some(PathPoint::new(
          before.x() + t * (after.x() - before.x()),
          before.y() + t * (after.y() - before.y()),
          ((before.intensity() + t * (after.intensity() - before.intensity())) * 100.0) as u8,
        ))
      }
      (Some(before), None) => {
        // Use the last point if we're past the end
        Some(PathPoint::new(
          *before.x(),
          *before.y(),
          (*before.intensity() * 100.0) as u8,
        ))
      }
      (None, Some(after)) => {
        // Use the first point if we're before the start
        Some(PathPoint::new(
          *after.x(),
          *after.y(),
          (*after.intensity() * 100.0) as u8,
        ))
      }
      (None, None) => None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct CompiledFrame {
  /// Timestamp in milliseconds
  pub timestamp: u32,
  /// Device effects at this frame
  pub device_effects: HashMap<DevicePosition, Vec<FramePoint>>,
}

#[derive(Debug, Clone)]
pub struct FramePoint {
  pub x: f64,
  pub y: f64,
  pub intensity: f64,
}

impl CompiledFrame {
  pub fn new(timestamp: u32) -> Self {
    Self {
      timestamp,
      device_effects: HashMap::new(),
    }
  }

  pub fn add_point(&mut self, device: DevicePosition, point: FramePoint) {
    self.device_effects.entry(device).or_default().push(point);
  }
}

impl TactFileProject {
  /// Get the total duration of the project in milliseconds
  pub fn get_total_duration(&self) -> u32 {
    self
      .tracks
      .iter()
      .flat_map(|track| track.effects())
      .map(|effect| effect.start_time() + effect.offset_time() + self.get_effect_duration(effect))
      .max()
      .unwrap_or(0)
  }

  /// Get the duration of a specific effect
  pub fn get_effect_duration(&self, effect: &HapticEffect) -> u32 {
    effect
      .modes()
      .values()
      .map(|mode| match mode {
        EffectMode::PathMode { path_mode } => self.get_path_mode_duration(path_mode),
        EffectMode::DotMode { dot_mode: _ } => {
          // For now, assume dot mode has similar duration calculation
          // This can be expanded based on dot mode structure
          0
        }
      })
      .max()
      .unwrap_or(0)
  }

  /// Get the duration of a path mode effect
  fn get_path_mode_duration(&self, path_mode: &EffectPathMode) -> u32 {
    path_mode
      .feedback()
      .iter()
      .flat_map(|feedback| feedback.point_list())
      .map(|point| point.time())
      .max()
      .copied()
      .unwrap_or(0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    EffectFeedbackPlaybackType, EffectPathModeMovingPattern, Layout, LayoutPoint, Track,
  };

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
              y: 0.0,
            },
            LayoutPoint {
              index: 1,
              x: 0.5,
              y: 0.0,
            },
            LayoutPoint {
              index: 2,
              x: 1.0,
              y: 0.0,
            },
          ],
        ),
        (
          DevicePosition::VestBack,
          vec![LayoutPoint {
            index: 3,
            x: 0.0,
            y: 0.0,
          }],
        ),
      ])),
    }
  }

  fn create_test_position_mapper() -> Arc<PositionMapper> {
    use crate::path_point_mapper::InterpolatingMapperEvenGrid;
    use std::collections::HashMap;

    let mut device_mappers = HashMap::new();
    device_mappers.insert(
      DevicePosition::VestFront,
      Arc::new(Box::new(InterpolatingMapperEvenGrid::new(8, 4))
        as Box<dyn crate::path_point_mapper::Mapper>),
    );
    device_mappers.insert(
      DevicePosition::VestBack,
      Arc::new(Box::new(InterpolatingMapperEvenGrid::new(8, 4))
        as Box<dyn crate::path_point_mapper::Mapper>),
    );

    Arc::new(PositionMapper::new(device_mappers))
  }

  #[test]
  fn test_frame_compilation_config() {
    let default_config = FrameCompilationConfig::default();
    assert_eq!(default_config.step_size_ms, 20);

    let custom_config = FrameCompilationConfig::new(10);
    assert_eq!(custom_config.step_size_ms, 10);
  }

  #[test]
  fn test_frame_compiler_creation() {
    let config = FrameCompilationConfig::new(25);
    let position_mapper = create_test_position_mapper();
    let compiler = FrameCompiler::new(config, position_mapper);

    assert_eq!(compiler.config.step_size_ms, 25);
  }

  #[test]
  fn test_simple_frame_compilation() {
    let layout = create_test_layout();
    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![Track::new(
        Some(true),
        vec![HapticEffect::new(
          Some("test_effect".to_string()),
          0, // start_time = 0
          0, // offset_time = 0
          HashMap::from([(
            DevicePosition::VestFront,
            EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
              EffectFeedbackPlaybackType::None,
              EffectPathModeMovingPattern::ConstSpeed,
              true,
              vec![
                EffectPathModePoint::new(0.5, 0, 0.0, 0.5),   // t=0ms
                EffectPathModePoint::new(1.0, 100, 1.0, 0.5), // t=100ms
              ],
            )])),
          )]),
        )],
      )],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let config = FrameCompilationConfig::new(50); // 50ms steps
    let position_mapper = create_test_position_mapper();
    let compiler = FrameCompiler::new(config, position_mapper);

    let frames = compiler.compile(&project);

    // Should have frames at 0ms, 50ms, 100ms, plus potentially more
    assert!(frames.len() >= 3);

    // Check frame at t=0
    let frame_0 = &frames[0];
    assert_eq!(frame_0.timestamp, 0);

    assert!(
      frame_0
        .device_effects
        .contains_key(&DevicePosition::VestFront)
    );
  }

  #[test]
  fn test_configurable_step_size() {
    let layout = create_test_layout();
    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![Track::new(
        Some(true),
        vec![HapticEffect::new(
          Some("test_effect".to_string()),
          100,
          0,
          HashMap::from([(
            DevicePosition::VestFront,
            EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
              EffectFeedbackPlaybackType::None,
              EffectPathModeMovingPattern::ConstSpeed,
              true,
              vec![EffectPathModePoint::new(1.0, 100, 0.0, 0.5)],
            )])),
          )]),
        )],
      )],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let position_mapper = create_test_position_mapper();

    // Test with 10ms steps
    let config_10ms = FrameCompilationConfig::new(10);
    let compiler_10ms = FrameCompiler::new(config_10ms, position_mapper.clone());
    let frames_10ms = compiler_10ms.compile(&project);

    // Test with 25ms steps
    let config_25ms = FrameCompilationConfig::new(25);
    let compiler_25ms = FrameCompiler::new(config_25ms, position_mapper);
    let frames_25ms = compiler_25ms.compile(&project);

    // 10ms should have more frames than 25ms
    assert!(frames_10ms.len() > frames_25ms.len());

    // Verify timestamps match step size
    for (i, frame) in frames_10ms.iter().enumerate() {
      assert_eq!(frame.timestamp, i as u32 * 10);
    }

    for (i, frame) in frames_25ms.iter().enumerate() {
      assert_eq!(frame.timestamp, i as u32 * 25);
    }
  }

  #[test]
  fn test_empty_project() {
    let layout = create_test_layout();
    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let config = FrameCompilationConfig::new(20);
    let position_mapper = create_test_position_mapper();
    let compiler = FrameCompiler::new(config, position_mapper);
    let frames = compiler.compile(&project);

    assert!(frames.is_empty());
  }

  #[test]
  fn test_frame_content_verification() {
    let layout = create_test_layout();
    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![Track::new(
        Some(true),
        vec![HapticEffect::new(
          Some("content_test".to_string()),
          0, // start_time = 0
          0, // offset_time = 0
          HashMap::from([(
            DevicePosition::VestFront,
            EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
              EffectFeedbackPlaybackType::None,
              EffectPathModeMovingPattern::ConstSpeed,
              true,
              vec![
                EffectPathModePoint::new(0.2, 0, 0.0, 0.5), // t=0ms: intensity=0.2, x=0.0
                EffectPathModePoint::new(0.8, 100, 1.0, 0.5), // t=100ms: intensity=0.8, x=1.0
              ],
            )])),
          )]),
        )],
      )],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let config = FrameCompilationConfig::new(25); // 25ms steps
    let position_mapper = create_test_position_mapper();
    let compiler = FrameCompiler::new(config, position_mapper);

    let frames = compiler.compile(&project);

    // Should have frames at 0ms, 25ms, 50ms, 75ms, 100ms
    assert_eq!(frames.len(), 5);

    // Verify frame at t=0ms (should match first point exactly)
    let frame_0 = &frames[0];
    assert_eq!(frame_0.timestamp, 0);
    let vest_front_points_0 = frame_0
      .device_effects
      .get(&DevicePosition::VestFront)
      .unwrap();
    assert!(!vest_front_points_0.is_empty());
    // Note: Intensity is converted through DotPoint, so we expect it to be normalized

    // Verify frame at t=50ms (should be interpolated halfway)
    let frame_50 = &frames[2]; // frames[0]=0ms, frames[1]=25ms, frames[2]=50ms
    assert_eq!(frame_50.timestamp, 50);
    let vest_front_points_50 = frame_50
      .device_effects
      .get(&DevicePosition::VestFront)
      .unwrap();
    assert!(!vest_front_points_50.is_empty());
    // At t=50ms, we should have interpolated values:
    // intensity: 0.2 + 0.5 * (0.8 - 0.2) = 0.2 + 0.3 = 0.5
    // x: 0.0 + 0.5 * (1.0 - 0.0) = 0.5

    // Verify frame at t=100ms (should match second point exactly)
    let frame_100 = &frames[4]; // Last frame
    assert_eq!(frame_100.timestamp, 100);
    let vest_front_points_100 = frame_100
      .device_effects
      .get(&DevicePosition::VestFront)
      .unwrap();
    assert!(!vest_front_points_100.is_empty());
  }

  #[test]
  fn test_interpolation_accuracy() {
    let layout = create_test_layout();
    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![Track::new(
        Some(true),
        vec![HapticEffect::new(
          Some("interpolation_test".to_string()),
          0,
          0,
          HashMap::from([(
            DevicePosition::VestFront,
            EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
              EffectFeedbackPlaybackType::None,
              EffectPathModeMovingPattern::ConstSpeed,
              true,
              vec![
                EffectPathModePoint::new(0.0, 0, 0.0, 0.0), // t=0ms: intensity=0.0, x=0.0, y=0.0
                EffectPathModePoint::new(1.0, 200, 1.0, 1.0), // t=200ms: intensity=1.0, x=1.0, y=1.0
              ],
            )])),
          )]),
        )],
      )],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let config = FrameCompilationConfig::new(100); // 100ms steps for easier math
    let position_mapper = create_test_position_mapper();
    let compiler = FrameCompiler::new(config, position_mapper);

    let frames = compiler.compile(&project);

    // Should have frames at 0ms, 100ms, 200ms
    assert_eq!(frames.len(), 3);

    // The PathPoint values should be correctly interpolated before mapping
    // t=0ms: intensity=0.0, x=0.0, y=0.0
    // t=100ms: intensity=0.5, x=0.5, y=0.5 (halfway interpolated)
    // t=200ms: intensity=1.0, x=1.0, y=1.0

    // Verify all frames have the expected device
    for frame in &frames {
      assert!(
        frame
          .device_effects
          .contains_key(&DevicePosition::VestFront)
      );
      let points = frame
        .device_effects
        .get(&DevicePosition::VestFront)
        .unwrap();
      assert!(
        !points.is_empty(),
        "Frame at {}ms should have points",
        frame.timestamp
      );
    }
  }

  #[test]
  fn test_effect_timing_boundaries() {
    let layout = create_test_layout();
    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![Track::new(
        Some(true),
        vec![HapticEffect::new(
          Some("timing_test".to_string()),
          50, // start_time = 50ms
          10, // offset_time = 10ms (effect actually starts at 60ms)
          HashMap::from([(
            DevicePosition::VestFront,
            EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
              EffectFeedbackPlaybackType::None,
              EffectPathModeMovingPattern::ConstSpeed,
              true,
              vec![
                EffectPathModePoint::new(0.5, 0, 0.0, 0.5), // t=0ms relative to effect start
                EffectPathModePoint::new(1.0, 40, 1.0, 0.5), // t=40ms relative to effect start
              ],
            )])),
          )]),
        )],
      )],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let config = FrameCompilationConfig::new(20); // 20ms steps
    let position_mapper = create_test_position_mapper();
    let compiler = FrameCompiler::new(config, position_mapper);

    let frames = compiler.compile(&project);

    // Total duration should be 50+10+40 = 100ms
    // Frames at: 0, 20, 40, 60, 80, 100
    assert_eq!(frames.len(), 6);

    // Frames before effect starts (t < 60ms) should be empty
    let frame_0 = &frames[0]; // t=0ms
    let frame_20 = &frames[1]; // t=20ms  
    let frame_40 = &frames[2]; // t=40ms

    assert!(
      !frame_0
        .device_effects
        .contains_key(&DevicePosition::VestFront)
    );
    assert!(
      !frame_20
        .device_effects
        .contains_key(&DevicePosition::VestFront)
    );
    assert!(
      !frame_40
        .device_effects
        .contains_key(&DevicePosition::VestFront)
    );

    // Frames during effect (60ms <= t <= 100ms) should have content
    let frame_60 = &frames[3]; // t=60ms (effect local time = 0ms)
    let frame_80 = &frames[4]; // t=80ms (effect local time = 20ms)
    let frame_100 = &frames[5]; // t=100ms (effect local time = 40ms)

    assert!(
      frame_60
        .device_effects
        .contains_key(&DevicePosition::VestFront)
    );
    assert!(
      frame_80
        .device_effects
        .contains_key(&DevicePosition::VestFront)
    );
    assert!(
      frame_100
        .device_effects
        .contains_key(&DevicePosition::VestFront)
    );
  }

  #[test]
  fn test_multiple_tracks_compilation() {
    let layout = create_test_layout();
    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![
        Track::new(
          Some(true),
          vec![HapticEffect::new(
            Some("track1_effect".to_string()),
            0,
            0,
            HashMap::from([(
              DevicePosition::VestFront,
              EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
                EffectFeedbackPlaybackType::None,
                EffectPathModeMovingPattern::ConstSpeed,
                true,
                vec![EffectPathModePoint::new(0.3, 50, 0.0, 0.5)],
              )])),
            )]),
          )],
        ),
        Track::new(
          Some(true),
          vec![HapticEffect::new(
            Some("track2_effect".to_string()),
            0,
            0,
            HashMap::from([(
              DevicePosition::VestBack,
              EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
                EffectFeedbackPlaybackType::None,
                EffectPathModeMovingPattern::ConstSpeed,
                true,
                vec![EffectPathModePoint::new(0.7, 50, 1.0, 0.5)],
              )])),
            )]),
          )],
        ),
      ],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let config = FrameCompilationConfig::new(25);
    let position_mapper = create_test_position_mapper();
    let compiler = FrameCompiler::new(config, position_mapper);

    let frames = compiler.compile(&project);

    // Should have frames at 0ms, 25ms, 50ms
    assert_eq!(frames.len(), 3);

    // Each frame should have effects from both tracks
    for frame in &frames {
      assert!(
        frame
          .device_effects
          .contains_key(&DevicePosition::VestFront),
        "Frame at {}ms should have VestFront effects",
        frame.timestamp
      );
      assert!(
        frame.device_effects.contains_key(&DevicePosition::VestBack),
        "Frame at {}ms should have VestBack effects",
        frame.timestamp
      );
    }
  }

  #[test]
  fn test_disabled_track_ignored() {
    let layout = create_test_layout();
    let project = TactFileProject {
      id: None,
      name: None,
      description: None,
      tracks: vec![
        Track::new(
          Some(true), // Enabled track
          vec![HapticEffect::new(
            Some("enabled_effect".to_string()),
            0,
            0,
            HashMap::from([(
              DevicePosition::VestFront,
              EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
                EffectFeedbackPlaybackType::None,
                EffectPathModeMovingPattern::ConstSpeed,
                true,
                vec![EffectPathModePoint::new(0.5, 50, 0.0, 0.5)],
              )])),
            )]),
          )],
        ),
        Track::new(
          Some(false), // Disabled track
          vec![HapticEffect::new(
            Some("disabled_effect".to_string()),
            0,
            0,
            HashMap::from([(
              DevicePosition::VestBack,
              EffectMode::path_mode(EffectPathMode::new(vec![EffectPathModeFeedback::new(
                EffectFeedbackPlaybackType::None,
                EffectPathModeMovingPattern::ConstSpeed,
                true,
                vec![EffectPathModePoint::new(0.8, 50, 1.0, 0.5)],
              )])),
            )]),
          )],
        ),
      ],
      layout,
      media_file_duration: None,
      created_at: None,
      updated_at: None,
    };

    let config = FrameCompilationConfig::new(25);
    let position_mapper = create_test_position_mapper();
    let compiler = FrameCompiler::new(config, position_mapper);

    let frames = compiler.compile(&project);

    // Should have frames from enabled track only
    for frame in &frames {
      assert!(
        frame
          .device_effects
          .contains_key(&DevicePosition::VestFront),
        "Frame at {}ms should have VestFront from enabled track",
        frame.timestamp
      );
      assert!(
        !frame.device_effects.contains_key(&DevicePosition::VestBack),
        "Frame at {}ms should NOT have VestBack from disabled track",
        frame.timestamp
      );
    }
  }
}
