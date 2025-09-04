use crate::DevicePosition;
use derivative::Derivative;
use getset::Getters;

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

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PathPoint {
  x: f64,
  y: f64,
  intensity: u32,

  #[cfg_attr(feature = "serde", serde(default = "default_motor_count"))]
  motor_count: usize,
}

pub const fn default_motor_count() -> usize {
  3
}
