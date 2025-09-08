use derivative::Derivative;
use getset::{Getters, MutGetters, WithSetters};

#[derive(Derivative, Getters, MutGetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub(crate)")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct DotPoint {
  /// On-device index of the dot.
  index: usize,

  /// Intensity of the dot. Range: `0 .. =100`
  intensity: u32,
}

impl DotPoint {
  pub const MAX_INTENSITY: u32 = 100;

  pub fn new(index: usize, intensity: u32) -> Self {
    Self {
      index,
      intensity: intensity.min(Self::MAX_INTENSITY),
    }
  }
}

#[derive(Derivative, Getters, WithSetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PathPoint {
  /// X coordinate of the point. Range: `0.0 .. =1.0`
  x: f64,

  /// Y coordinate of the point. Range: `0.0 .. =1.0`
  y: f64,

  /// Intensity of the point. Range: `0 .. =100`
  intensity: u8,

  /// Numbers of motors, to interpolate the point between.
  /// I've only seen `3`
  #[getset(get = "pub", set_with = "pub")]
  #[cfg_attr(feature = "serde", serde(default = "default_motor_count"))]
  motor_count: usize,
}

impl PathPoint {
  pub const MAX_INTENSITY: u8 = 100;

  pub fn new(x: f64, y: f64, intensity: u8) -> Self {
    Self {
      x,
      y,
      intensity: intensity.min(Self::MAX_INTENSITY),
      motor_count: default_motor_count(),
    }
  }
}

pub const fn default_motor_count() -> usize {
  3
}
