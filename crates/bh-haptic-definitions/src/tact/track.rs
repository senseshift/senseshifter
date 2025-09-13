use derivative::Derivative;
use getset::{Getters, MutGetters};

use crate::HapticEffect;

#[derive(Derivative, Getters, MutGetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub")]
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
