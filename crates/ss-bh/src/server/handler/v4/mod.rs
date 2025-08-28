use derivative::Derivative;
use getset::Getters;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::{Message as TungsteniteMessage};
use tokio_tungstenite::tungstenite::handshake::server::Request as TungsteniteRequest;
use tracing::{*, Instrument};
use bh_sdk::v4::SdkEncryptedMessage;
use crate::server::handler::common::fetch_haptic_definitions;

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
}

fn get_server_key() -> String {
  "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAsTEzseYMqrG9JXbTnfWVUqV95k0yzkqiixokLTNrfCnpn7FEhZqPd6VBVXbZlAyijrjBio4ptKneFn8KDP0mYRvR3LTiEuIRoKmG9dlPBNQ4w0OafncYY+fM99WuWRSz1P7Ai/D4cAeoUkVZzKLNuePqayQueRV3i3xS76KBpHso5yJHl7Oheco4EZa+mWhjTiDbKhVTe/Mt+Xy/Amm6EmlVIyJLdOKdclBvEdc4Ja+fZ/yGqt61s2PHQGvFUZzW5GNmmYbQ5NM3zcNeudmICgHSQ/C2s5P5tALdpdE9F5z3GIv6oYG/RvkdtaMDn3GAEgrhcBDgJaABEe9mIx7RDQIDAQAB".to_string()
}

impl V4HandlerBuilder {
  pub fn new() -> Self {
    Self {
    }
  }

  pub fn build(
    self,
    request: TungsteniteRequest,
    sender: UnboundedSender<TungsteniteMessage>,
  ) -> crate::Result<V4Handler> {
    let uri = request.uri();
    let query = uri.query().unwrap_or_default();
    let app_definitions: AppDefinitions = serde_qs::from_str(query)?;

    // let decoded_server_key = base64::decode(get_server_key())?;
    // let data = &decoded_server_key[..];

    let encrypted_message = SdkEncryptedMessage::new(
      "ServerKey".to_string(),
      Some(get_server_key()),
      // fill 588/2 = 294 bytes with 'zero' and then base64 encode
      // Some(base64::encode(vec![0u8; 294])),
      None,
    );
    sender.send(TungsteniteMessage::Text(serde_json::to_string(&encrypted_message)?.into()))?;

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
          eprintln!("Failed to fetch haptic definitions: {}", e);
        });
      }.instrument(info_span!("fetch_haptic_definitions"))
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