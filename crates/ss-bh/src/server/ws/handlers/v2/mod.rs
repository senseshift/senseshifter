use getset::Getters;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use tracing::*;

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[get = "pub"]
pub struct AppContext {
  app_id: String,
  app_name: String,
}

pub struct FeedbackHandlerBuilder {
  app_ctx: AppContext,
  cancellation_token: Option<CancellationToken>,
}

impl FeedbackHandlerBuilder {
  pub fn new(app_ctx: AppContext) -> Self {
    Self {
      app_ctx,
      cancellation_token: None,
    }
  }

  pub async fn build(self) -> anyhow::Result<FeedbackHandler> {
    let _cancellation_token = self.cancellation_token.unwrap_or_default();

    Ok(FeedbackHandler {
      app_ctx: self.app_ctx,
    })
  }
}

pub struct FeedbackHandler {
  app_ctx: AppContext,
}

impl FeedbackHandler {
  #[instrument(skip(self, _msg))]
  pub async fn handle_message(&self, _msg: &str) -> anyhow::Result<()> {
    unimplemented!()
  }

  #[instrument(skip(self))]
  pub async fn handle_close(&self) -> anyhow::Result<()> {
    Ok(())
  }
}
