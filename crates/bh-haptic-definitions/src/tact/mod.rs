mod track;
mod effect;
mod frame;

pub use track::*;
pub use effect::*;
pub use frame::*;

use std::collections::HashMap;
use derivative::Derivative;
use derive_more::with_trait::Display;
use getset::Getters;

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
  #[cfg_attr(feature = "serde", serde(default, deserialize_with = "serde_handy::de::from_str_num_to_opt_string"))]
  id: Option<String>,
  name: String,
  description: Option<String>,

  tracks: Vec<Track>,
  layout: Layout,

  media_file_duration: Option<f64>,

  created_at: Option<u64>,
  updated_at: Option<u64>,
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Layout {
  name: String,

  r#type: String,

  /// List of points to reference in tracks.
  layouts: Option<HashMap<String, Vec<LayoutPoint>>>,
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
