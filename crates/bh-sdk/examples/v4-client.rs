use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use bh_sdk::v4::SdkEncryptedMessage;
use futures_util::{SinkExt, StreamExt};
use rand::RngCore;
use rsa::{
    pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info};
use bh_sdk::v4::SdkEncryptedMessageType::{SdkClientKey, SdkData};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SdkMessage {
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ts: Option<u64>,
}

struct CryptoClient {
    aes_key: Option<[u8; 32]>,
}

impl CryptoClient {
    fn new() -> Self {
        Self { aes_key: None }
    }

    fn set_aes_key(&mut self, key: [u8; 32]) {
        self.aes_key = Some(key);
    }

    fn encrypt_aes_gcm(&self, plaintext: &str) -> Result<String> {
        let key = self
            .aes_key
            .ok_or_else(|| anyhow!("AES key not established"))?;

        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let mut iv = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut iv);
        let nonce = Nonce::from_slice(&iv);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Concatenate: IV (12 bytes) + ciphertext + tag (already included in ciphertext by aes_gcm)
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&iv);
        result.extend_from_slice(&ciphertext);

        Ok(STANDARD.encode(result))
    }

    fn decrypt_aes_gcm(&self, data_b64: &str) -> Result<String> {
        let key = self
            .aes_key
            .ok_or_else(|| anyhow!("AES key not established"))?;

        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        let raw = STANDARD.decode(data_b64)?;
        if raw.len() < 12 + 16 {
            return Err(anyhow!("Cipher too short"));
        }

        let iv = &raw[0..12];
        let ciphertext = &raw[12..];

        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let nonce = Nonce::from_slice(iv);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(String::from_utf8(plaintext)?)
    }

    fn encrypt_rsa_pkcs1v15(&self, public_key_pem: &str, data: &[u8]) -> Result<String> {
        // Try parsing as SPKI first, then PKCS1 if that fails
        let public_key = RsaPublicKey::from_public_key_pem(public_key_pem)
            .or_else(|_| RsaPublicKey::from_pkcs1_pem(public_key_pem))?;

        let mut rng = rand::thread_rng();
        let encrypted = public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)?;

        Ok(STANDARD.encode(encrypted))
    }
}

fn spki_b64_to_pem(b64: &str) -> String {
    let body = b64
        .chars()
        .collect::<Vec<char>>()
        .chunks(64)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
        body
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <ws_url>", args[0]);
        eprintln!("Example: {} \"ws://127.0.0.1:15881/v4/feedback?workspace_id=test&api_key=test&version=1.0\"", args[0]);
        return Ok(());
    }

    let ws_url = &args[1];
    info!("Connecting to {}", ws_url);

    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let mut crypto_client = CryptoClient::new();

    // Handle incoming messages
    let write_handle = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<SdkEncryptedMessage>(&text) {
                        Ok(sdk_msg) => {
                            info!("← {}: {}", sdk_msg.r#type(), if text.len() > 200 { format!("{}…", &text[..200]) } else { text.to_string() });

                            match sdk_msg.r#type() {
                                bh_sdk::v4::SdkEncryptedMessageType::ServerKey => {
                                    if let Some(key_b64) = sdk_msg.key() {
                                        // Convert server key to PEM format
                                        let server_pem = spki_b64_to_pem(key_b64);
                                        info!("ServerKey SPKI length: {}", key_b64.len());

                                        // Generate AES key and encrypt it with server's public key
                                        let mut aes_key = [0u8; 32];
                                        rand::thread_rng().fill_bytes(&mut aes_key);
                                        
                                        match crypto_client.encrypt_rsa_pkcs1v15(&server_pem, &aes_key) {
                                            Ok(encrypted_key_b64) => {
                                                crypto_client.set_aes_key(aes_key);
                                                info!("AES key: {}", hex::encode(aes_key));

                                                // Send SdkClientKey
                                                let client_key_msg = SdkEncryptedMessage::new(
                                                    SdkClientKey,
                                                    Some(encrypted_key_b64),
                                                    None,
                                                );
                                                
                                                if let Ok(json) = serde_json::to_string(&client_key_msg) {
                                                    if let Err(e) = write.send(Message::Text(json.clone().into())).await {
                                                        error!("Failed to send SdkClientKey: {}", e);
                                                    } else {
                                                        info!("→ {}", json);
                                                    }
                                                }

                                                // Send a ping after establishing AES
                                                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                                                
                                                let ping_msg = SdkMessage {
                                                    r#type: "SdkPingAll".to_string(),
                                                    message: Some("".to_string()),
                                                    ts: None,
                                                };

                                                if let Ok(ping_json) = serde_json::to_string(&ping_msg) {
                                                    if let Ok(encrypted_data) = crypto_client.encrypt_aes_gcm(&ping_json) {
                                                        let encrypted_msg = SdkEncryptedMessage::new(
                                                            SdkData,
                                                            None,
                                                            Some(encrypted_data),
                                                        );
                                                        
                                                        if let Ok(json) = serde_json::to_string(&encrypted_msg) {
                                                            if let Err(e) = write.send(Message::Text(json.clone().into())).await {
                                                                error!("Failed to send ping: {}", e);
                                                            } else {
                                                                info!("→ SdkData (ping)");
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => error!("RSA encryption failed: {}", e),
                                        }
                                    }
                                }
                                SdkData => {
                                    if let Some(encrypted_data) = sdk_msg.data() {
                                        match crypto_client.decrypt_aes_gcm(encrypted_data) {
                                            Ok(plaintext) => {
                                                info!("SdkData ← {}", plaintext);
                                                
                                                // Handle specific message types
                                                if let Ok(parsed_msg) = serde_json::from_str::<SdkMessage>(&plaintext) {
                                                    if parsed_msg.r#type == "SdkPongAll" {
                                                        info!("Got pong: {:?}", parsed_msg);
                                                    }
                                                }
                                            }
                                            Err(e) => error!("AES decrypt error: {}", e),
                                        }
                                    }
                                }
                                _ => {
                                    info!("Other message type: {}", sdk_msg.r#type());
                                }
                            }
                        }
                        Err(_) => {
                            info!("← non-JSON: {}", if text.len() > 200 { format!("{}…", &text[..200]) } else { text.to_string() });
                        }
                    }
                }
                Ok(Message::Close(close_frame)) => {
                    info!("WebSocket closed: {:?}", close_frame);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    info!("WebSocket connected. Press Ctrl+C to exit.");
    
    // Wait for Ctrl+C
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down...");
        }
        _ = write_handle => {
            info!("Connection closed");
        }
    }

    Ok(())
}