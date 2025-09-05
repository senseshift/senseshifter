use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct AppContext {
  workspace_id: String,
  api_key: String,
  version: Option<String>,
}
