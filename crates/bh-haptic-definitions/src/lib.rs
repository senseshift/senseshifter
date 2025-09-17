mod audio;
mod device;
pub mod path_point_mapper;
mod tact;
mod traits;

pub use audio::*;
pub use device::*;
pub use tact::*;

use anyhow::*;
use derivative::Derivative;
use getset::{Getters, WithSetters};
use tracing::*;

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SdkApiResponseV3<T> {
  status: bool,
  code: i64,
  error_message: Option<String>,
  timestamp: u64,
  message: Option<T>,
}

impl<T> SdkApiResponseV3<T> {
  pub fn new(
    status: bool,
    code: i64,
    error_message: Option<String>,
    timestamp: u64,
    message: Option<T>,
  ) -> Self {
    Self {
      status,
      code,
      error_message,
      timestamp,
      message,
    }
  }
}

#[cfg(feature = "client")]
pub async fn fetch_haptic_definitions(
  app_id: &str,
  api_key: &str,
  // version: String,
) -> Result<HapticDefinitionsMessage, Error> {
  let url = format!(
    "https://sdk-apis.bhaptics.com/api/v1/haptic-definitions/workspace-v3/latest?latest-version={}&api-key={}&app-id={}",
    -1, api_key, app_id
  );

  info!("Fetching haptic definitions from URL: {}", url);

  let response = reqwest::get(url).await?;
  let response_body = response
    .json::<SdkApiResponseV3<HapticDefinitionsMessage>>()
    .await
    .context("Failed to parse haptic definitions response")?;

  response_body
    .message
    .context("No message in haptic definitions response")
}

#[derive(Derivative, Getters, WithSetters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub", set_with = "pub")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HapticDefinitionsMessage {
  id: Option<String>,
  create_time: Option<u64>,
  name: Option<String>,
  description: Option<String>,
  creator: Option<String>,
  workspace_id: Option<String>,
  version: Option<i64>,
  disable_validation: Option<bool>,
  category_options: Option<Vec<String>>,

  #[cfg_attr(feature = "serde", serde(default))]
  haptic_mappings: Vec<HapticDefinitionMapping>,
}

impl HapticDefinitionsMessage {
  pub fn new(haptic_mappings: Vec<HapticDefinitionMapping>) -> Self {
    Self {
      id: None,
      create_time: None,
      name: None,
      description: None,
      creator: None,
      workspace_id: None,
      version: None,
      disable_validation: None,
      category_options: None,
      haptic_mappings,
    }
  }
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HapticDefinitionMapping {
  enable: Option<bool>,

  #[cfg_attr(
    feature = "serde",
    serde(deserialize_with = "serde_handy::de::to_opt_f64")
  )]
  intensity: Option<f64>,
  key: String,
  category: Option<String>,
  description: Option<String>,
  update_time: Option<u64>,

  event_time: u32,

  #[cfg_attr(feature = "serde", serde(default))]
  tact_file_patterns: Vec<HapticDefinitionTactFilePattern>,

  #[cfg_attr(feature = "serde", serde(default))]
  audio_file_patterns: Vec<HapticDefinitionAudioFilePattern>,
}
