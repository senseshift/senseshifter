use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::*;

use super::{HandlerBuilder, MessageHandler, v3};
use crate::server::{HapticManagerCommand, HapticManagerEvent};
use bh_sdk::v4::SdkEncryptedMessage;

use anyhow::anyhow;
use base64::{Engine, engine::general_purpose::STANDARD};
use bh_sdk::v3::{SdkMessage, SdkRequestAuthMessage};
use getset::WithSetters;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rsa::pkcs8::EncodePublicKey;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde_json;

mod crypto;
use crypto::CryptoContext;

/// Shared helper function for encrypting V3 messages and sending as V4
async fn encrypt_and_send_v3_message(
  crypto: &CryptoContext,
  v3_json: &str,
  ws_sender: &mpsc::UnboundedSender<Message>,
) -> anyhow::Result<()> {
  debug!("Encrypting V3 → V4: {}", v3_json);

  // Encrypt V3 JSON using crypto context
  let encrypted_data = crypto.encrypt_aes_gcm(v3_json).await?;

  // Wrap in V4 SdkData message
  let v4_msg = SdkEncryptedMessage::sdk_data(encrypted_data);
  let json = serde_json::to_string(&v4_msg)?;

  // Send via WebSocket
  ws_sender.send(Message::Text(json.into()))?;
  Ok(())
}

/// V4 AppContext with encryption support (extended from V3)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppContext {
  pub workspace_id: String,
  pub api_key: String,
  pub version: Option<String>,
  pub device_id: Option<String>,
}

impl From<AppContext> for v3::AppContext {
  fn from(val: AppContext) -> Self {
    v3::AppContext::new(val.workspace_id, val.api_key, val.version)
  }
}

impl From<&AppContext> for v3::AppContext {
  fn from(val: &AppContext) -> Self {
    v3::AppContext::new(
      val.workspace_id.clone(),
      val.api_key.clone(),
      val.version.clone(),
    )
  }
}

/// V4 Handler Builder - accepts a pre-built V3 handler
#[derive(WithSetters)]
pub struct FeedbackHandlerBuilder {
  app_ctx: AppContext,
  v3_handler: Option<v3::FeedbackHandler>,
  v3_message_rx: Option<mpsc::UnboundedReceiver<Message>>,
  #[getset(set_with = "pub")]
  private_key: Option<RsaPrivateKey>,
  ws_sender: mpsc::UnboundedSender<Message>,
  cancellation_token: Option<CancellationToken>,
}

impl HandlerBuilder for FeedbackHandlerBuilder {
  type Handler = FeedbackHandler;
  type Context = AppContext;

  fn new(
    context: Self::Context,
    _command_sender: mpsc::Sender<HapticManagerCommand>, // V3 handler already has this
    ws_sender: mpsc::UnboundedSender<Message>,
  ) -> Self {
    // This will be called from a custom upgrade function that builds V3 externally
    Self {
      app_ctx: context,
      v3_handler: None,    // Will be set by with_v3_handler
      v3_message_rx: None, // Will be set by with_v3_message_receiver
      private_key: None,
      ws_sender,
      cancellation_token: None,
    }
  }

  fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
    self.cancellation_token = Some(token);
    self
  }

  async fn build(self) -> anyhow::Result<Self::Handler> {
    let v3_handler = self
      .v3_handler
      .ok_or_else(|| anyhow::anyhow!("V3 handler not provided"))?;

    let v3_message_rx = self
      .v3_message_rx
      .ok_or_else(|| anyhow::anyhow!("V3 message receiver not provided"))?;

    let mut rng = ChaCha20Rng::from_rng(&mut rand::rng());

    let private_key = match self.private_key {
      Some(key) => key,
      None => {
        warn!("Generating new RSA key. This is a slow operation, please configure persistent key");

        RsaPrivateKey::new(&mut rng, 2048)
          .map_err(|err| anyhow!("Failed to generate RSA key: {}", err))?
      }
    };

    let public_key = RsaPublicKey::from(&private_key);
    let public_key_der = public_key.to_public_key_der()?;

    let crypto = CryptoContext::new(rng, private_key, public_key_der);

    // Start V3 message interceptor task with shared crypto context
    FeedbackHandler::start_v3_message_interceptor(
      v3_message_rx,
      self.ws_sender.clone(),
      crypto.clone(),
    );

    Ok(FeedbackHandler {
      app_ctx: self.app_ctx,
      v3_handler,
      ws_sender: self.ws_sender,
      crypto,
    })
  }
}

impl FeedbackHandlerBuilder {
  /// Set the V3 handler for composition
  pub fn with_v3_handler(mut self, v3_handler: v3::FeedbackHandler) -> Self {
    self.v3_handler = Some(v3_handler);
    self
  }

  /// Set the V3 message receiver for intercepting outgoing V3 messages
  pub fn with_v3_message_receiver(
    mut self,
    v3_message_rx: mpsc::UnboundedReceiver<Message>,
  ) -> Self {
    self.v3_message_rx = Some(v3_message_rx);
    self
  }
}

/// V4 Encrypted Handler - wraps V3 handler with encryption layer
pub struct FeedbackHandler {
  app_ctx: AppContext,
  v3_handler: v3::FeedbackHandler, // Wrapped V3 handler
  ws_sender: mpsc::UnboundedSender<Message>,
  crypto: CryptoContext,
}

impl MessageHandler for FeedbackHandler {
  type Context = AppContext;
  type Builder = FeedbackHandlerBuilder;

  #[instrument(skip(self))]
  async fn handle_connection_opened(&mut self) -> anyhow::Result<()> {
    info!(
      "V4 Encrypted WebSocket connection opened for workspace: {}",
      self.app_ctx.workspace_id
    );

    let public_key_spki_b64 = STANDARD.encode(self.crypto.public_key_der().as_bytes());
    let server_key_msg = SdkEncryptedMessage::server_key(public_key_spki_b64);

    self.send_raw_message(&server_key_msg).await?;

    self.v3_handler.handle_connection_opened().await
  }

  #[instrument(skip(self, msg))]
  async fn handle_text_message(&mut self, msg: &str) -> anyhow::Result<()> {
    let sdk_msg: SdkEncryptedMessage = serde_json::from_str(msg)
      .map_err(|e| anyhow::anyhow!("Failed to parse V4 encrypted message: {}", e))?;

    match sdk_msg {
      SdkEncryptedMessage::SdkClientKey { key: encrypted_key } => {
        // Decrypt RSA-encrypted AES key and store it
        let decrypted_key = self.crypto.decrypt_client_key_pkcs1v15(&encrypted_key)?;
        self.crypto.set_aes_key(decrypted_key);

        info!("V4 encryption handshake completed successfully");

        // sending message to the v3 handler, to fetch haptic definitions, since the v4 clients do not send these messages themselves
        // todo: I'm not sure what these empty values are doing, so I just left them empty, since in my implementation they do not do anything
        self
          .v3_handler
          .handle_sdk_message(&SdkMessage::SdkRequestAuth(SdkRequestAuthMessage::new(
            "".to_string(),
            self.app_ctx.workspace_id.clone(),
            "".to_string(),
            "".to_string(),
            self.app_ctx.api_key.clone(),
          )))
          .await
      }

      SdkEncryptedMessage::SdkData {
        data: encrypted_data,
      } => {
        if !self.is_handshake_complete() {
          return Err(anyhow::anyhow!("Received data before handshake complete"));
        }

        // Decrypt the V4 message to get V3 JSON
        let v3_json = self.crypto.decrypt_aes_gcm(&encrypted_data)?;
        debug!("Decrypted V4 → V3: {}", v3_json);

        // Forward decrypted message to wrapped V3 handler
        self.v3_handler.handle_text_message(&v3_json).await
      }

      SdkEncryptedMessage::ServerKey { .. } => {
        warn!("Received unexpected ServerKey message from client");
        Ok(())
      }
    }
  }

  #[instrument(skip(self, _data))]
  async fn handle_binary_message(&mut self, _data: &[u8]) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("Binary messages not supported"))
  }

  #[instrument(skip(self))]
  async fn handle_close(&mut self) -> anyhow::Result<()> {
    info!("V4 Encrypted WebSocket connection closing");
    // Delegate to wrapped handler
    self.v3_handler.handle_close().await
  }

  #[instrument(skip(self, event))]
  async fn handle_haptic_event(&mut self, event: &HapticManagerEvent) -> anyhow::Result<()> {
    if !self.is_handshake_complete() {
      return Ok(()); // Skip events until encryption is established
    }

    self.v3_handler.handle_haptic_event(event).await
  }
}

impl FeedbackHandler {
  /// Check if the encryption handshake is complete (AES key established)
  fn is_handshake_complete(&self) -> bool {
    self.crypto.get_aes_key().is_some()
  }

  /// Helper to send encrypted messages back to the client
  #[allow(dead_code)]
  #[instrument(skip(self, v3_json))]
  async fn send_encrypted_v3_message(&mut self, v3_json: &str) -> anyhow::Result<()> {
    if !self.is_handshake_complete() {
      return Err(anyhow::anyhow!(
        "Cannot send encrypted message before handshake"
      ));
    }

    // Use shared encrypt+send logic
    encrypt_and_send_v3_message(&self.crypto, v3_json, &self.ws_sender).await
  }

  async fn send_raw_message(&self, msg: &impl Serialize) -> anyhow::Result<()> {
    let json = serde_json::to_string(msg)?;
    self.ws_sender.send(Message::Text(json.into()))?;
    Ok(())
  }

  /// Start background task to intercept V3 messages and encrypt them
  fn start_v3_message_interceptor(
    mut v3_message_rx: mpsc::UnboundedReceiver<Message>,
    ws_sender: mpsc::UnboundedSender<Message>,
    crypto: CryptoContext,
  ) {
    tokio::spawn(async move {
      while let Some(v3_message) = v3_message_rx.recv().await {
        match v3_message {
          Message::Text(text) => {
            debug!("Intercepted V3 text message: {}", text);

            // Use the same shared encrypt+send logic as the handler!
            if let Err(e) = encrypt_and_send_v3_message(&crypto, &text, &ws_sender).await {
              error!("Failed to encrypt and send V3→V4 message: {}", e);
            }
          }
          Message::Binary(_) => {
            warn!("V3 binary messages not supported for encryption");
          }
          _ => {
            debug!("Ignoring non-content V3 message type");
          }
        }
      }

      info!("V3→V4 message interceptor task completed");
    });
  }
}
