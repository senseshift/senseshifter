use anyhow::anyhow;

use base64::{Engine, prelude::BASE64_STANDARD};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs8::EncodePublicKey;

pub(crate) struct CryptoContext {
  rng: ChaCha20Rng,

  private_key: RsaPrivateKey,
  public_key_spki_b64: String,

  aes_key: Option<[u8; 32]>,
}

impl CryptoContext {
  pub fn new(
    rng: ChaCha20Rng,
    private_key: RsaPrivateKey,
  ) -> crate::Result<Self> {
    let mut result = Self {
      rng,
      private_key,
      public_key_spki_b64: "".to_string(),
      aes_key: None,
    };

    result.update_public_key()?;

    Ok(result)
  }

  fn update_public_key(&mut self) -> crate::Result<&Self> {
    let public_key = RsaPublicKey::from(&self.private_key);
    let public_key_der = public_key.to_public_key_der()?;
    let public_key_spki_b64 = BASE64_STANDARD.encode(public_key_der.as_bytes());

    self.public_key_spki_b64 = public_key_spki_b64;

    Ok(self)
  }

  pub fn set_public_key(&mut self, public_key_spki_b64: String) -> crate::Result<&Self> {
    self.public_key_spki_b64 = public_key_spki_b64;
    self.update_public_key()?;
    Ok(self)
  }

  pub fn set_aes_key(&mut self, aes_key: [u8; 32]) -> &Self {
    self.aes_key = Some(aes_key);
    self
  }
}

impl CryptoContext {
  const IV_LENGTH: usize = 12;

  pub fn encrypt_aes_gcm(&mut self, plaintext: &str) -> crate::Result<String> {
    use aes_gcm::{
      aead::{Aead, KeyInit},
      Aes256Gcm, Nonce,
    };

    let key = self
      .aes_key
      .ok_or_else(|| anyhow!("AES key not established"))?;

    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let iv = {
      let mut buf = [0u8; Self::IV_LENGTH];
      self.rng.fill_bytes(&mut buf);
      buf
    };
    let nonce = Nonce::from_slice(&iv);

    let ciphertext = cipher
      .encrypt(nonce, plaintext.as_bytes())
      .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    let mut result = Vec::with_capacity(iv.len() + ciphertext.len());
    result.extend_from_slice(&iv);
    result.extend_from_slice(&ciphertext);

    Ok(BASE64_STANDARD.encode(result))
  }

  pub fn decrypt_aes_gcm(&self, data_b64: &str) -> anyhow::Result<String> {
    use aes_gcm::{
      aead::{Aead, KeyInit},
      Aes256Gcm, Nonce,
    };

    let key = self
      .aes_key
      .ok_or_else(|| anyhow!("AES key not established"))?;

    let raw = BASE64_STANDARD.decode(data_b64)?;
    if raw.len() < Self::IV_LENGTH + 16 {
      return Err(anyhow!("Cipher too short"));
    }

    let iv = &raw[..Self::IV_LENGTH];
    let ciphertext = &raw[Self::IV_LENGTH..];

    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let nonce = Nonce::from_slice(iv);

    let plaintext = cipher
      .decrypt(nonce, ciphertext)
      .map_err(|e| anyhow!("Decryption failed: {}", e))?;

    Ok(String::from_utf8(plaintext)?)
  }
}