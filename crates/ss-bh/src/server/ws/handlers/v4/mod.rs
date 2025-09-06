use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::*;

use super::{HandlerBuilder, MessageHandler, v3};
use crate::server::{HapticManagerCommand, HapticManagerEvent};
use bh_sdk::v4::SdkEncryptedMessage;

use aes_gcm::{
  Aes256Gcm, Nonce,
  aead::{Aead, KeyInit},
};
use anyhow::anyhow;
use base64::{Engine, engine::general_purpose::STANDARD};
use getset::WithSetters;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rsa::pkcs8::{Document, EncodePublicKey};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

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
  rng: ChaCha20Rng,
  private_key: RsaPrivateKey,
  public_key_der: Document,
  aes_key: Option<[u8; 32]>,
}

impl CryptoContext {
  fn set_aes_key(&mut self, key: [u8; 32]) {
    self.aes_key = Some(key);
  }

  fn decrypt_client_key_pkcs1v15(&self, encrypted_key_b64: &str) -> anyhow::Result<[u8; 32]> {
    // Decode base64 encrypted key
    let encrypted_key = STANDARD
      .decode(encrypted_key_b64)
      .map_err(|e| anyhow!("Failed to decode base64 encrypted key: {}", e))?;

    // Decrypt using RSA PKCS1v15
    let decrypted = self
      .private_key
      .decrypt(Pkcs1v15Encrypt, &encrypted_key)
      .map_err(|e| anyhow!("Failed to decrypt RSA key: {}", e))?;

    // Ensure we have exactly 32 bytes for AES-256
    if decrypted.len() != 32 {
      return Err(anyhow!(
        "Invalid AES key length: expected 32 bytes, got {}",
        decrypted.len()
      ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&decrypted);
    Ok(key)
  }

  fn encrypt_aes_gcm(&mut self, plaintext: &str) -> anyhow::Result<String> {
    let key = self
      .aes_key
      .ok_or_else(|| anyhow::anyhow!("AES key not established"))?;

    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let mut iv = [0u8; 12];
    self.rng.fill_bytes(&mut iv);
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

    let crypto = Arc::new(Mutex::new(CryptoContext {
      rng,
      private_key,
      public_key_der,
      aes_key: None,
    }));

    // Start V3 message interceptor task
    FeedbackHandler::start_v3_message_interceptor(
      v3_message_rx,
      self.ws_sender.clone(),
      Arc::clone(&crypto),
    );

    Ok(FeedbackHandler {
      app_ctx: self.app_ctx,
      v3_handler,
      ws_sender: self.ws_sender,
      crypto,
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
  crypto: Arc<Mutex<CryptoContext>>,
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

    let public_key_spki_b64 = {
      let crypto = self.crypto.lock().await;
      STANDARD.encode(crypto.public_key_der.as_bytes())
    };
    let server_key_msg = SdkEncryptedMessage::server_key(public_key_spki_b64);

    self.send_raw_message(&server_key_msg).await
  }

  #[instrument(skip(self, msg))]
  async fn handle_text_message(&mut self, msg: &str) -> anyhow::Result<()> {
    let sdk_msg: SdkEncryptedMessage = serde_json::from_str(msg)
      .map_err(|e| anyhow::anyhow!("Failed to parse V4 encrypted message: {}", e))?;

    match sdk_msg {
      SdkEncryptedMessage::SdkClientKey { key: encrypted_key } => {
        info!("Received client AES key, establishing encryption");

        // Decrypt RSA-encrypted AES key and store it
        let mut crypto = self.crypto.lock().await;
        let decrypted_key = crypto.decrypt_client_key_pkcs1v15(&encrypted_key)?;
        crypto.set_aes_key(decrypted_key);
        self.handshake_complete = true;

        info!("V4 encryption handshake completed successfully");
        Ok(())
      }

      SdkEncryptedMessage::SdkData {
        data: encrypted_data,
      } => {
        if !self.handshake_complete {
          return Err(anyhow::anyhow!("Received data before handshake complete"));
        }

        // Decrypt the V4 message to get V3 JSON
        let v3_json = self.crypto.lock().await.decrypt_aes_gcm(&encrypted_data)?;
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
  /// Helper to send encrypted messages back to the client
  #[instrument(skip(self, v3_json))]
  async fn send_encrypted_v3_message(&mut self, v3_json: &str) -> anyhow::Result<()> {
    if !self.handshake_complete {
      return Err(anyhow::anyhow!(
        "Cannot send encrypted message before handshake"
      ));
    }

    // Encrypt V3 JSON
    let encrypted_data = self.crypto.lock().await.encrypt_aes_gcm(v3_json)?;

    // Wrap in V4 SdkData message
    let v4_msg = SdkEncryptedMessage::sdk_data(encrypted_data);

    self.send_raw_message(&v4_msg).await
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
    crypto: Arc<Mutex<CryptoContext>>,
  ) {
    tokio::spawn(async move {
      info!("V3→V4 message interceptor task started");

      while let Some(v3_message) = v3_message_rx.recv().await {
        match v3_message {
          Message::Text(text) => {
            debug!("Intercepted V3 text message: {}", text);

            // Encrypt V3 JSON and send as V4 message
            let mut crypto_guard = crypto.lock().await;
            match crypto_guard.encrypt_aes_gcm(&text) {
              Ok(encrypted_data) => {
                drop(crypto_guard); // Release lock early

                let v4_msg = SdkEncryptedMessage::sdk_data(encrypted_data);
                match serde_json::to_string(&v4_msg) {
                  Ok(json) => {
                    if let Err(e) = ws_sender.send(Message::Text(json.into())) {
                      error!("Failed to send encrypted V3→V4 message: {}", e);
                      break;
                    }
                  }
                  Err(e) => {
                    error!("Failed to serialize V4 message: {}", e);
                  }
                }
              }
              Err(e) => {
                error!("Failed to encrypt V3 message: {}", e);
              }
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
