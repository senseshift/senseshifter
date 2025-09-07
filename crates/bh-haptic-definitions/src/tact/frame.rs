use crate::DevicePosition;
use derivative::Derivative;
use getset::{Getters, WithSetters};

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HapticFrame {
  duration_millis: u32,
  position_type: DevicePosition,

  dot_points: Vec<DotPoint>,
  path_points: Vec<PathPoint>,
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct DotPoint {
  index: u32,
  intensity: u32,
}

impl DotPoint {
  pub fn new(index: u32, intensity: u32) -> Self {
    Self { index, intensity }
  }
}

#[derive(Derivative, Getters, WithSetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PathPoint {
  x: f64,
  y: f64,
  intensity: u32,

  #[getset(get = "pub", set_with = "pub")]
  #[cfg_attr(feature = "serde", serde(default = "default_motor_count"))]
  motor_count: usize,
}

impl PathPoint {
  pub fn new(x: f64, y: f64, intensity: u32) -> Self {
    Self {
      x,
      y,
      intensity,
      motor_count: default_motor_count(),
    }
  }
}

pub const fn default_motor_count() -> usize {
  3
}
