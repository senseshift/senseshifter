mod track;
mod effect;

pub use track::*;
pub use effect::*;

use std::collections::HashMap;
use derivative::Derivative;
use getset::Getters;

use crate::{DevicePosition, DeviceType};

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
#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
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
  id: Option<String>,
  name: String,
  description: Option<String>,

  tracks: Vec<Track>,
  layout: Layout,

  media_file_duration: Option<u32>,

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

  r#type: DeviceType,

  /// List of points to reference in tracks.
  layouts: Option<HashMap<DevicePosition, Vec<LayoutPoint>>>,
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LayoutPoint {
  index: u32,

  #[derivative(Hash="ignore")]
  x: f64,

  #[derivative(Hash="ignore")]
  y: f64,
}
