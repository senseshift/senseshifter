mod crypto;

use crypto::*;
use crate::server::handler::common::fetch_haptic_definitions;
use bh_sdk::v4::SdkEncryptedMessage;

use derivative::Derivative;
use getset::Getters;
use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rsa::{RsaPrivateKey};

use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::handshake::server::Request as TungsteniteRequest;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

use tracing::{Instrument, *};
use anyhow::anyhow;

#[derive(Derivative, Getters, Serialize, Deserialize)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
pub struct AppDefinitions {
  workspace_id: String,
  api_key: String,
  #[serde(default = "default_app_version")]
  version: String,
}

fn default_app_version() -> String {
  "-1".to_string()
}

pub struct V4HandlerBuilder {
  private_key: Option<RsaPrivateKey>
}

impl V4HandlerBuilder {
  pub fn new() -> Self {
    Self {
      private_key: None,
    }
  }

  #[allow(dead_code)]
  pub fn private_key(mut self, private_key: RsaPrivateKey) -> Self {
    self.private_key = Some(private_key);
    self
  }

  pub fn build(
    self,
    request: TungsteniteRequest,
    sender: UnboundedSender<TungsteniteMessage>,
  ) -> crate::Result<V4Handler> {
    let uri = request.uri();
    let query = uri.query().unwrap_or_default();
    let app_definitions: AppDefinitions = serde_qs::from_str(query)?;

    let mut rng = ChaCha20Rng::from_rng(&mut rand::rng());

    let private_key = match self.private_key {
      Some(key) => key,
      None => {
        info!("Generating new RSA key");

        RsaPrivateKey::new(&mut rng, 2048)?
      },
    };

    let crypto_context = CryptoContext::new(rng, private_key)?;
    let crypto_context = Arc::new(Mutex::new(crypto_context));

    let handler = V4Handler {
      app_definitions: app_definitions.clone(),
      sender,
    };

    let _join_task = tokio::spawn(
      async move {
        // Here you can perform any asynchronous initialization if needed.
        // For example, fetching haptic definitions
        fetch_haptic_definitions(
          app_definitions.workspace_id.clone(),
          app_definitions.api_key.clone(),
          // app_definitions.version.clone(),
        ).await.unwrap_or_else(|e| {
          error!("Failed to fetch haptic definitions: {}", e);
        });
      }.instrument(info_span!("fetch_haptic_definitions_task"))
    );

    Ok(handler)
  }
}

pub struct V4Handler {
  app_definitions: AppDefinitions,
  sender: UnboundedSender<TungsteniteMessage>,
}

impl V4Handler {
  #[tracing::instrument(skip(self, message))]
  pub async fn handle_message(&self, message: TungsteniteMessage) -> crate::Result<()> {
    // info!("Received message: {:#?}", message);
    Ok(())
  }
}