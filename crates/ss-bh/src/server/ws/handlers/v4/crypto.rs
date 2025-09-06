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
  #[allow(dead_code)]
  pub fn get_aes_key_watcher(&self) -> watch::Receiver<Option<[u8; 32]>> {
    self.inner.aes_key_rx.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand::SeedableRng;
  use rsa::pkcs8::EncodePublicKey;
  use rsa::{RsaPrivateKey, RsaPublicKey};

  /// Create a test CryptoContext with a small RSA key for testing
  fn create_test_crypto_context() -> CryptoContext {
    let mut rng = ChaCha20Rng::from_seed([42u8; 32]); // Deterministic for tests
    let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("Failed to generate RSA key");
    let public_key = RsaPublicKey::from(&private_key);
    let public_key_der = public_key
      .to_public_key_der()
      .expect("Failed to encode public key");

    CryptoContext::new(rng, private_key, public_key_der)
  }

  /// Generate a test AES key
  fn generate_test_aes_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    for (i, byte) in key.iter_mut().enumerate() {
      *byte = i as u8;
    }
    key
  }

  #[test]
  fn test_aes_key_management() {
    let crypto = create_test_crypto_context();
    let test_key = generate_test_aes_key();

    // Initially no key
    assert!(crypto.get_aes_key().is_none());

    // Set key
    crypto.set_aes_key(test_key);

    // Should now have the key
    assert_eq!(crypto.get_aes_key(), Some(test_key));
  }

  #[tokio::test]
  async fn test_encrypt_decrypt_roundtrip() {
    let crypto = create_test_crypto_context();
    let test_key = generate_test_aes_key();
    crypto.set_aes_key(test_key);

    let plaintext = "Hello, V4 encryption world! 🔒";

    // Encrypt
    let encrypted = crypto
      .encrypt_aes_gcm(plaintext)
      .await
      .expect("Failed to encrypt");

    // Should be different from plaintext
    assert_ne!(encrypted, plaintext);

    // Decrypt
    let decrypted = crypto
      .decrypt_aes_gcm(&encrypted)
      .expect("Failed to decrypt");

    // Should match original plaintext
    assert_eq!(decrypted, plaintext);
  }

  #[tokio::test]
  async fn test_encrypt_without_key_fails() {
    let crypto = create_test_crypto_context();
    // No AES key set

    let result = crypto.encrypt_aes_gcm("test message").await;

    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("AES key not established")
    );
  }

  #[test]
  fn test_decrypt_without_key_fails() {
    let crypto = create_test_crypto_context();
    // No AES key set

    let result = crypto.decrypt_aes_gcm("fake_encrypted_data");

    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("AES key not established")
    );
  }

  #[tokio::test]
  async fn test_encrypt_produces_different_ciphertexts() {
    let crypto = create_test_crypto_context();
    let test_key = generate_test_aes_key();
    crypto.set_aes_key(test_key);

    let plaintext = "Same message";

    // Encrypt the same message twice
    let encrypted1 = crypto
      .encrypt_aes_gcm(plaintext)
      .await
      .expect("Failed to encrypt first time");
    let encrypted2 = crypto
      .encrypt_aes_gcm(plaintext)
      .await
      .expect("Failed to encrypt second time");

    // Should produce different ciphertexts (due to random IV)
    assert_ne!(encrypted1, encrypted2);

    // But both should decrypt to the same plaintext
    let decrypted1 = crypto
      .decrypt_aes_gcm(&encrypted1)
      .expect("Failed to decrypt first");
    let decrypted2 = crypto
      .decrypt_aes_gcm(&encrypted2)
      .expect("Failed to decrypt second");

    assert_eq!(decrypted1, plaintext);
    assert_eq!(decrypted2, plaintext);
  }

  #[test]
  fn test_decrypt_invalid_base64_fails() {
    let crypto = create_test_crypto_context();
    let test_key = generate_test_aes_key();
    crypto.set_aes_key(test_key);

    let result = crypto.decrypt_aes_gcm("invalid_base64!!!");

    assert!(result.is_err());
  }

  #[test]
  fn test_decrypt_too_short_data_fails() {
    let crypto = create_test_crypto_context();
    let test_key = generate_test_aes_key();
    crypto.set_aes_key(test_key);

    // Create valid base64 but too short for IV + auth tag
    let short_data = base64::engine::general_purpose::STANDARD.encode([1, 2, 3]);

    let result = crypto.decrypt_aes_gcm(&short_data);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cipher too short"));
  }

  #[test]
  fn test_rsa_encrypt_decrypt_roundtrip() {
    let crypto = create_test_crypto_context();

    // Generate a test AES key
    let original_key = generate_test_aes_key();

    // Encrypt with RSA (simulate client sending key)
    let public_key = RsaPublicKey::from(&crypto.inner.private_key);
    let mut rng = rand::rng();
    let encrypted_key = public_key
      .encrypt(&mut rng, rsa::Pkcs1v15Encrypt, &original_key)
      .expect("Failed to RSA encrypt");

    // Encode to base64
    let encrypted_key_b64 = base64::engine::general_purpose::STANDARD.encode(encrypted_key);

    // Decrypt using our method
    let decrypted_key = crypto
      .decrypt_client_key_pkcs1v15(&encrypted_key_b64)
      .expect("Failed to decrypt client key");

    // Should match original
    assert_eq!(decrypted_key, original_key);
  }

  #[test]
  fn test_rsa_decrypt_invalid_base64_fails() {
    let crypto = create_test_crypto_context();

    let result = crypto.decrypt_client_key_pkcs1v15("invalid_base64!!!");

    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("Failed to decode base64")
    );
  }

  #[test]
  fn test_rsa_decrypt_wrong_key_length_fails() {
    let crypto = create_test_crypto_context();

    // Create valid RSA encrypted data but with wrong key length (not 32 bytes)
    let public_key = RsaPublicKey::from(&crypto.inner.private_key);
    let mut rng = rand::rng();
    let wrong_length_key = [42u8; 16]; // Only 16 bytes, should be 32

    let encrypted = public_key
      .encrypt(&mut rng, rsa::Pkcs1v15Encrypt, &wrong_length_key)
      .expect("Failed to encrypt");
    let encrypted_b64 = base64::engine::general_purpose::STANDARD.encode(encrypted);

    let result = crypto.decrypt_client_key_pkcs1v15(&encrypted_b64);

    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("Invalid AES key length")
    );
  }

  #[tokio::test]
  async fn test_concurrent_encryption() {
    let crypto = create_test_crypto_context();
    let test_key = generate_test_aes_key();
    crypto.set_aes_key(test_key);

    let plaintext = "Concurrent encryption test";

    // Spawn multiple encryption tasks concurrently
    let mut tasks = Vec::new();
    for i in 0..10 {
      let crypto_clone = crypto.clone();
      let message = format!("{} - message {}", plaintext, i);
      let message_clone = message.clone();

      let task = tokio::spawn(async move { crypto_clone.encrypt_aes_gcm(&message_clone).await });

      tasks.push((task, message));
    }

    // Wait for all tasks and verify results
    for (task, original_message) in tasks {
      let encrypted = task.await.expect("Task failed").expect("Encryption failed");

      let decrypted = crypto
        .decrypt_aes_gcm(&encrypted)
        .expect("Decryption failed");

      assert_eq!(decrypted, original_message);
    }
  }

  #[tokio::test]
  async fn test_aes_key_watcher() {
    let crypto = create_test_crypto_context();
    let mut watcher = crypto.get_aes_key_watcher();

    // Initially should be None
    assert_eq!(*watcher.borrow(), None);

    // Set a key
    let test_key = generate_test_aes_key();
    crypto.set_aes_key(test_key);

    // Watcher should be notified
    assert!(watcher.changed().await.is_ok());
    assert_eq!(*watcher.borrow(), Some(test_key));

    // Set another key
    let test_key2 = [255u8; 32];
    crypto.set_aes_key(test_key2);

    // Watcher should see the new key
    assert!(watcher.changed().await.is_ok());
    assert_eq!(*watcher.borrow(), Some(test_key2));
  }
}
