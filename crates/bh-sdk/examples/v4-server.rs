use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use bh_sdk::v4::{SdkEncryptedMessage, SdkEncryptedMessageType};
use futures_util::{SinkExt, StreamExt};
use rand::RngCore;
use rsa::{
    pkcs8::EncodePublicKey, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::{
    accept_async,
    tungstenite::Message,
};
use tracing::{error, info, warn};
use bh_sdk::v4::SdkEncryptedMessageType::{SdkData, ServerKey};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SdkMessage {
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ts: Option<u64>,
}

struct ClientConnection {
    aes_key: Option<[u8; 32]>,
    conn_id: String,
}

impl ClientConnection {
    fn new(conn_id: String) -> Self {
        Self {
            aes_key: None,
            conn_id,
        }
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
}

struct ServerState {
    private_key: RsaPrivateKey,
    public_key_spki_b64: String,
    clients: Arc<Mutex<HashMap<String, ClientConnection>>>,
}

impl ServerState {
    fn new() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
        let public_key = RsaPublicKey::from(&private_key);
        
        // Get public key in SPKI format and convert to base64
        let public_key_der = public_key.to_public_key_der()?;
        let public_key_spki_b64 = STANDARD.encode(public_key_der.as_bytes());

        Ok(Self {
            private_key,
            public_key_spki_b64,
            clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn decrypt_client_key_pkcs1v15(&self, encrypted_b64: &str) -> Result<[u8; 32]> {
        // Try both standard base64 and base64url
        let candidates = vec![
            STANDARD.decode(encrypted_b64)?,
            STANDARD.decode(&encrypted_b64.replace('-', "+").replace('_', "/"))?,
        ];

        for encrypted_data in candidates {
            if let Ok(decrypted) = self.private_key.decrypt(Pkcs1v15Encrypt, &encrypted_data) {
                if decrypted.len() == 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&decrypted);
                    return Ok(key);
                } else {
                    warn!("Decrypted key length: {} (expected 32)", decrypted.len());
                }
            }
        }
        
        Err(anyhow!("RSA v1.5 decrypt failed"))
    }

    async fn handle_client(&self, stream: TcpStream, addr: SocketAddr) -> Result<()> {
        let conn_id = format!("{:x}", rand::random::<u32>());
        info!("[{}] New client from {}", conn_id, addr);

        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        // Add client to state
        {
            let mut clients = self.clients.lock().await;
            clients.insert(conn_id.clone(), ClientConnection::new(conn_id.clone()));
        }

        // Send ServerKey
        let server_key_msg = SdkEncryptedMessage::new(
            ServerKey,
            Some(self.public_key_spki_b64.clone()),
            None,
        );
        
        let server_key_json = serde_json::to_string(&server_key_msg)?;
        write.send(Message::Text(server_key_json.into())).await?;

        // Handle messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(sdk_msg) = serde_json::from_str::<SdkEncryptedMessage>(&text) {
                        match sdk_msg.r#type() {
                            SdkEncryptedMessageType::SdkClientKey => {
                                if let Some(encrypted_key_b64) = sdk_msg.key() {
                                    info!("[{}] SdkClientKey base64 length: {}", conn_id, encrypted_key_b64.len());
                                    
                                    match self.decrypt_client_key_pkcs1v15(encrypted_key_b64) {
                                        Ok(aes_key) => {
                                            info!("[{}] AES key: {}", conn_id, hex::encode(aes_key));
                                            
                                            // Update client with AES key
                                            {
                                                let mut clients = self.clients.lock().await;
                                                if let Some(client) = clients.get_mut(&conn_id) {
                                                    client.set_aes_key(aes_key);
                                                }
                                            }

                                            // Send welcome message
                                            let welcome_msg = SdkMessage {
                                                r#type: "SdkServerHello".to_string(),
                                                message: Some("hi".to_string()),
                                                ts: None,
                                            };
                                            
                                            let welcome_json = serde_json::to_string(&welcome_msg)?;
                                            
                                            let clients = self.clients.lock().await;
                                            if let Some(client) = clients.get(&conn_id) {
                                                if let Ok(encrypted_data) = client.encrypt_aes_gcm(&welcome_json) {
                                                    let encrypted_msg = SdkEncryptedMessage::new(
                                                        SdkData,
                                                        None,
                                                        Some(encrypted_data),
                                                    );
                                                    
                                                    let encrypted_json = serde_json::to_string(&encrypted_msg)?;
                                                    write.send(Message::Text(encrypted_json.into())).await?;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("[{}] SdkClientKey decrypt failed: {}", conn_id, e);
                                        }
                                    }
                                }
                            }
                            SdkData => {
                                if let Some(encrypted_data) = sdk_msg.data() {
                                    let clients = self.clients.lock().await;
                                    if let Some(client) = clients.get(&conn_id) {
                                        match client.decrypt_aes_gcm(encrypted_data) {
                                            Ok(plaintext) => {
                                                info!("[{}] SdkData ← {}", conn_id, plaintext);
                                                
                                                // Handle ping-pong
                                                if let Ok(parsed_msg) = serde_json::from_str::<SdkMessage>(&plaintext) {
                                                    if parsed_msg.r#type == "SdkPingAll" {
                                                        let pong_msg = SdkMessage {
                                                            r#type: "SdkPongAll".to_string(),
                                                            message: None,
                                                            ts: Some(std::time::SystemTime::now()
                                                                .duration_since(std::time::UNIX_EPOCH)
                                                                .unwrap()
                                                                .as_millis() as u64),
                                                        };
                                                        
                                                        let pong_json = serde_json::to_string(&pong_msg)?;
                                                        
                                                        if let Ok(encrypted_pong) = client.encrypt_aes_gcm(&pong_json) {
                                                            let encrypted_msg = SdkEncryptedMessage::new(
                                                                SdkData,
                                                                None,
                                                                Some(encrypted_pong),
                                                            );
                                                            
                                                            let encrypted_json = serde_json::to_string(&encrypted_msg)?;
                                                            write.send(Message::Text(encrypted_json.into())).await?;
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!("[{}] AES decrypt error: {}", conn_id, e);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {
                                info!("[{}] Other message type: {:?}", conn_id, sdk_msg.r#type());
                            }
                        }
                    } else {
                        info!("[{}] Non-JSON message: {}", conn_id, text);
                    }
                }
                Ok(Message::Close(close_frame)) => {
                    info!("[{}] WebSocket closed: {:?}", conn_id, close_frame);
                    break;
                }
                Err(e) => {
                    error!("[{}] WebSocket error: {}", conn_id, e);
                    break;
                }
                _ => {}
            }
        }

        // Remove client from state
        {
            let mut clients = self.clients.lock().await;
            clients.remove(&conn_id);
        }

        info!("[{}] Connection closed", conn_id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let server_state = ServerState::new()?;
    
    info!("\n[mock] SPKI (base64) sent to clients:\n{}", server_state.public_key_spki_b64);

    let addr = "127.0.0.1:15881";
    let listener = TcpListener::bind(&addr).await?;
    info!("Mock server listening on ws://{}/v4/feedback", addr);

    let server_state = Arc::new(server_state);
    
    while let Ok((stream, addr)) = listener.accept().await {
        let server_state_clone = Arc::clone(&server_state);
        
        tokio::spawn(async move {
            if let Err(e) = server_state_clone.handle_client(stream, addr).await {
                error!("Error handling client {}: {}", addr, e);
            }
        });
    }

    Ok(())
}