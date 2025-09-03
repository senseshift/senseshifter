mod dot;
mod path;

use dot::*;
use path::*;

use std::collections::HashMap;
use derivative::Derivative;
use getset::Getters;

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Effect {
  name: Option<String>,

  offset_time: Option<u32>,
  start_time: Option<u32>,

  modes: HashMap<String, EffectMode>,
}

/// From the clients I always receive both `dotMode` and `pathMode` fields, but from observation,
/// only the one, selected by the `mode` JSON field is used, so I assume we might optimize their
/// struct to enum.
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "mode", rename_all = "camelCase"))]
pub enum EffectMode {
  #[cfg_attr(feature = "serde", serde(rename = "DOT_MODE", rename_all = "camelCase"))]
  DotMode {
    dot_mode: EffectDotMode,
  },
  #[cfg_attr(feature = "serde", serde(rename = "PATH_MODE", rename_all = "camelCase"))]
  PathMode {
    path_mode: EffectPathMode,
  },
}

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EffectFeedbackPlaybackType {
  #[cfg_attr(feature = "serde", serde(rename = "NONE"))]
  None,
  #[cfg_attr(feature = "serde", serde(rename = "FADE_IN"))]
  FadeIn,
  #[cfg_attr(feature = "serde", serde(rename = "FADE_OUT"))]
  FadeOut,
  #[cfg_attr(feature = "serde", serde(rename = "FADE_IN_OUT"))]
  FadeInOut,
}