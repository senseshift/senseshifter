mod effect;
mod frame;
mod frame_compilation;
mod rotation;
mod track;

pub use effect::*;
pub use frame::*;
pub use frame_compilation::*;
pub use track::*;

use crate::DevicePosition;
use derivative::Derivative;
use derive_more::with_trait::Display;
use getset::{Getters, MutGetters};
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

#[derive(Derivative, Getters, MutGetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub")]
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
