use crate::DevicePosition;
use std::collections::HashMap;

use derivative::Derivative;
use getset::Getters;

#[cfg(feature = "serde")]
use serde_with::{base64::Base64, serde_as};

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", serde_as)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HapticDefinitionAudioFilePattern {
  pattern_id: String,
  snapshot_id: String,
  position: DevicePosition,
  clip: HapticDefinitionAudioFileClip,
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", serde_as)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HapticDefinitionAudioFileClip {
  id: String,
  name: String,
  version: i32,
  duration: u32,

  // todo: `Vec<u8>` is more like `[u8; 40]`
  #[cfg(feature = "serde")]
  #[serde_as(deserialize_as = "HashMap<_, Vec<Base64>>")]
  patterns: HashMap<DevicePosition, Vec<Vec<u8>>>,

  #[cfg(not(feature = "serde"))]
  patterns: HashMap<DevicePosition, Vec<Vec<u8>>>,
}
