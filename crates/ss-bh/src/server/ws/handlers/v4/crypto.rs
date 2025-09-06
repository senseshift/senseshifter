use std::sync::Arc;
use tokio::sync::{Mutex, watch};

use aes_gcm::{
  Aes256Gcm, Nonce,
  aead::{Aead, KeyInit},
};
use anyhow::anyhow;
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::RngCore;
use rand_chacha::ChaCha20Rng;
use rsa::pkcs8::Document;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

/// Crypto state for V4 encryption/decryption - async-friendly and shareable
#[derive(Debug, Clone)]
pub struct CryptoContext {
  inner: Arc<CryptoContextInner>,
}

#[derive(Debug)]
struct CryptoContextInner {
  rng: Mutex<ChaCha20Rng>,
  private_key: RsaPrivateKey,
  public_key_der: Document,
  aes_key_tx: watch::Sender<Option<[u8; 32]>>,
  aes_key_rx: watch::Receiver<Option<[u8; 32]>>,
}

impl CryptoContext {
  pub fn new(rng: ChaCha20Rng, private_key: RsaPrivateKey, public_key_der: Document) -> Self {
    let (aes_key_tx, aes_key_rx) = watch::channel(None);

    Self {
      inner: Arc::new(CryptoContextInner {
        rng: Mutex::new(rng),
        private_key,
        public_key_der,
        aes_key_tx,
        aes_key_rx,
      }),
    }
  }

  pub fn set_aes_key(&self, key: [u8; 32]) {
    let _ = self.inner.aes_key_tx.send(Some(key));
  }

  pub fn get_aes_key(&self) -> Option<[u8; 32]> {
    *self.inner.aes_key_rx.borrow()
  }

  pub fn decrypt_client_key_pkcs1v15(&self, encrypted_key_b64: &str) -> anyhow::Result<[u8; 32]> {
    // Decode base64 encrypted key
    let encrypted_key = STANDARD
      .decode(encrypted_key_b64)
      .map_err(|e| anyhow!("Failed to decode base64 encrypted key: {}", e))?;

    // Decrypt using RSA PKCS1v15
    let decrypted = self
      .inner
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

  /// Async-friendly encryption - works from any context (handler or interceptor)
  pub async fn encrypt_aes_gcm(&self, plaintext: &str) -> anyhow::Result<String> {
    let key = self
      .get_aes_key()
      .ok_or_else(|| anyhow::anyhow!("AES key not established"))?;

    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let mut iv = [0u8; 12];
    {
      let mut rng = self.inner.rng.lock().await;
      rng.fill_bytes(&mut iv);
    }
    let nonce = Nonce::from_slice(&iv);

    let ciphertext = cipher
      .encrypt(nonce, plaintext.as_bytes())
      .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&iv);
    result.extend_from_slice(&ciphertext);

    Ok(STANDARD.encode(result))
  }

  pub fn decrypt_aes_gcm(&self, data_b64: &str) -> anyhow::Result<String> {
    let key = self
      .get_aes_key()
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

  pub fn public_key_der(&self) -> &Document {
    &self.inner.public_key_der
  }

  /// Get a clone of the watch receiver for sharing with background tasks
  pub fn get_aes_key_watcher(&self) -> watch::Receiver<Option<[u8; 32]>> {
    self.inner.aes_key_rx.clone()
  }
}
