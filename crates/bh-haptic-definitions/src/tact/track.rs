use derivative::Derivative;
use getset::Getters;

use crate::HapticEffect;

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
