use derivative::Derivative;
use getset::Getters;

use crate::HapticEffect;
use crate::traits::ScaleEffect;

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Track {
  enable: Option<bool>,

  #[cfg_attr(feature = "serde", serde(default))]
  effects: Vec<HapticEffect>,
}

impl Track {
  pub fn new(enable: Option<bool>, effects: Vec<HapticEffect>) -> Self {
    Self { enable, effects }
  }
}

impl ScaleEffect for Track {
  #[inline]
  fn scale_effect(&mut self, duration_scale: f64, intensity: f64) {
    self
      .effects
      .iter_mut()
      .for_each(|e| e.scale_effect(duration_scale, intensity));
  }
}
