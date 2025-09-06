use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::*;

use super::{HandlerBuilder, MessageHandler, v3};
use crate::server::{HapticManagerCommand, HapticManagerEvent};
use bh_sdk::v4::{SdkEncryptedMessage, SdkEncryptedMessageType};

use aes_gcm::{
  Aes256Gcm, Nonce,
  aead::{Aead, KeyInit},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::RngCore;

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

/// Crypto state for V4 encryption/decryption
#[derive(Debug)]
struct CryptoContext {
  aes_key: Option<[u8; 32]>,
}

impl CryptoContext {
  fn new() -> Self {
    Self { aes_key: None }
  }

  fn set_aes_key(&mut self, key: [u8; 32]) {
    self.aes_key = Some(key);
  }

  fn encrypt_aes_gcm(&self, plaintext: &str) -> anyhow::Result<String> {
    let key = self
      .aes_key
      .ok_or_else(|| anyhow::anyhow!("AES key not established"))?;

    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let mut iv = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut iv);
    let nonce = Nonce::from_slice(&iv);

    let ciphertext = cipher
      .encrypt(nonce, plaintext.as_bytes())
      .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&iv);
    result.extend_from_slice(&ciphertext);

    Ok(STANDARD.encode(result))
  }

  fn decrypt_aes_gcm(&self, data_b64: &str) -> anyhow::Result<String> {
    let key = self
      .aes_key
      .ok_or_else(|| anyhow::anyhow!("AES key not established"))?;

    let raw = STANDARD.decode(data_b64)?;
    if raw.len() < 12 + 16 {
      return Err(anyhow::anyhow!("Cipher too short"));
    }

    let iv = &raw[0..12];
    let ciphertext = &raw[12..];

    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let nonce = Nonce::from_slice(iv);

    let plaintext = cipher
      .decrypt(nonce, ciphertext)
      .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(String::from_utf8(plaintext)?)
  }
}

/// V4 Handler Builder - accepts a pre-built V3 handler
pub struct FeedbackHandlerBuilder {
  app_ctx: AppContext,
  v3_handler: Option<v3::FeedbackHandler>,
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
      v3_handler: None, // Will be set by with_v3_handler
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

    Ok(FeedbackHandler {
      app_ctx: self.app_ctx,
      v3_handler,
      ws_sender: self.ws_sender,
      crypto: CryptoContext::new(),
      handshake_complete: false,
    })
  }
}

impl FeedbackHandlerBuilder {
  /// Set the V3 handler for composition
  pub fn with_v3_handler(mut self, v3_handler: v3::FeedbackHandler) -> Self {
    self.v3_handler = Some(v3_handler);
    self
  }
}

/// V4 Encrypted Handler - wraps V3 handler with encryption layer
pub struct FeedbackHandler {
  app_ctx: AppContext,
  v3_handler: v3::FeedbackHandler, // Wrapped V3 handler
  ws_sender: mpsc::UnboundedSender<Message>,
  crypto: CryptoContext,
  handshake_complete: bool,
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

    // Send server's public key to initiate encryption handshake
    // TODO: Implement RSA key generation and send ServerKey message
    // let server_key_msg = SdkEncryptedMessage::server_key(self.public_key_spki_b64.clone());
    // let json = serde_json::to_string(&server_key_msg)?;
    // self.ws_sender.send(Message::Text(json.into()))?;

    Ok(())
  }

  #[instrument(skip(self, msg))]
  async fn handle_text_message(&mut self, msg: &str) -> anyhow::Result<()> {
    info!("V4 Encrypted received message: {}", msg);

    let sdk_msg: SdkEncryptedMessage = serde_json::from_str(msg)
      .map_err(|e| anyhow::anyhow!("Failed to parse V4 encrypted message: {}", e))?;

    match sdk_msg.r#type() {
      SdkEncryptedMessageType::SdkClientKey => {
        if let Some(encrypted_key) = sdk_msg.key() {
          info!("Received client AES key, establishing encryption");
          // TODO: Decrypt RSA-encrypted AES key and store it
          // let decrypted_key = self.decrypt_client_key_pkcs1v15(encrypted_key)?;
          // self.crypto.set_aes_key(decrypted_key);
          self.handshake_complete = true;
        }
        Ok(())
      }

      SdkEncryptedMessageType::SdkData => {
        if !self.handshake_complete {
          return Err(anyhow::anyhow!("Received data before handshake complete"));
        }

        if let Some(encrypted_data) = sdk_msg.data() {
          // Decrypt the V4 message to get V3 JSON
          let v3_json = self.crypto.decrypt_aes_gcm(encrypted_data)?;
          info!("Decrypted V4 → V3: {}", v3_json);

          // Forward decrypted message to wrapped V3 handler
          self.v3_handler.handle_text_message(&v3_json).await?;
        }
        Ok(())
      }

      _ => {
        warn!(
          "Unhandled V4 encrypted message type: {:?}",
          sdk_msg.r#type()
        );
        Ok(())
      }
    }
  }

  #[instrument(skip(self, data))]
  async fn handle_binary_message(&mut self, data: &[u8]) -> anyhow::Result<()> {
    info!(
      "V4 Encrypted received binary message of {} bytes",
      data.len()
    );
    // V4 typically uses text messages for encrypted data
    Ok(())
  }

  #[instrument(skip(self))]
  async fn handle_close(&mut self) -> anyhow::Result<()> {
    info!("V4 Encrypted WebSocket connection closing");
    // Delegate to wrapped handler
    self.v3_handler.handle_close().await
  }

  #[instrument(skip(self, event))]
  async fn handle_haptic_event(&mut self, event: &HapticManagerEvent) -> anyhow::Result<()> {
    if !self.handshake_complete {
      return Ok(()); // Skip events until encryption is established
    }

    info!("V4 Encrypted received haptic event: {:?}", event);

    // Let V3 handler process the event (this might generate a response)
    // In a real implementation, we'd need to:
    // 1. Let V3 handler generate its response JSON
    // 2. Encrypt that JSON using AES-GCM
    // 3. Wrap in SdkEncryptedMessage::SdkData
    // 4. Send via self.ws_sender

    // For now, delegate to V3 handler
    self.v3_handler.handle_haptic_event(event).await
  }
}

impl FeedbackHandler {
  /// Helper to send encrypted messages back to client
  #[instrument(skip(self, v3_json))]
  async fn send_encrypted_v3_message(&self, v3_json: &str) -> anyhow::Result<()> {
    if !self.handshake_complete {
      return Err(anyhow::anyhow!(
        "Cannot send encrypted message before handshake"
      ));
    }

    // Encrypt V3 JSON
    let encrypted_data = self.crypto.encrypt_aes_gcm(v3_json)?;

    // Wrap in V4 SdkData message
    let v4_msg = SdkEncryptedMessage::sdk_data(encrypted_data);
    let v4_json = serde_json::to_string(&v4_msg)?;

    // Send to client
    self.ws_sender.send(Message::Text(v4_json.into()))?;
    info!("Sent encrypted V3 → V4: {}", v3_json);

    Ok(())
  }
}
