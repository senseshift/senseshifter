mod tact;
mod device;

pub use tact::*;
pub use device::*;

use anyhow::*;
use derivative::Derivative;
use getset::Getters;
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

impl <T> SdkApiResponseV3<T> {
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
  app_id: String,
  api_key: String,
  // version: String,
) -> Result<HapticDefinitionsMessage, Error> {
  let url = format!("https://sdk-apis.bhaptics.com/api/v1/haptic-definitions/workspace-v3/latest?latest-version={}&api-key={}&app-id={}", -1, api_key, app_id);

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

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
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

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HapticDefinitionMapping {
  enable: Option<bool>,
  
  #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_handy::de::to_opt_f64"))]
  intensity: Option<f64>,
  key: String,
  category: Option<String>,
  description: Option<String>,
  update_time: Option<u64>,

  event_time: u32,

  #[cfg_attr(feature = "serde", serde(default))]
  tact_file_patterns: Vec<HapticDefinitionTactFilePattern>,

  // #[serde(default)]
  // audio_file_patterns: Option<Vec<HapticDefinitionAudioFilePattern>>,
}