mod data;

use derivative::Derivative;
use reqwest::get;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use crate::server::haptic_definitions::data::HapticDefinitionsMessage;

#[cfg(any(feature = "v3", feature = "v4"))]
pub(crate) async fn fetch_haptic_definitions(
  app_id: String,
  api_key: String,
  // version: String,
) -> crate::Result<()> {
  let url = format!("https://sdk-apis.bhaptics.com/api/v1/haptic-definitions/workspace-v3/latest?latest-version={}&api-key={}&app-id={}", -1, api_key, app_id);

  info!("Fetching haptic definitions from URL: {}", url);

  let response = get(url).await?;

  debug!("Response Status: {}", response.status());

  Err(anyhow::anyhow!("Not implemented") )
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct HapticDefinitionsResponse {
  status: bool,
  code: i64,
  error_message: Option<String>,
  timestamp: u64,
  message: Option<HapticDefinitionsMessage>
}