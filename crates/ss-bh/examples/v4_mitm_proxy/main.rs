use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD};
use bh_sdk::v4::SdkEncryptedMessage;
use futures_util::{SinkExt, StreamExt};

use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

use rsa::{
  Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey,
  pkcs8::EncodePublicKey,
};

use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::{
  accept_hdr_async, connect_async,
  tungstenite::{
    Message,
    handshake::server::{Request, Response},
  },
};
use tracing::{error, info, warn};

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

  fn encrypt_aes_gcm(&self, plaintext: &str) -> Result<String> {
    let key = self
      .aes_key
      .ok_or_else(|| anyhow!("AES key not established"))?;

    use aes_gcm::{
      Aes256Gcm, Nonce,
      aead::{Aead, KeyInit},
    };

    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let mut iv = [0u8; 12];
    rand::rng().fill_bytes(&mut iv);
    let nonce = Nonce::from_slice(&iv);

    let ciphertext = cipher
      .encrypt(nonce, plaintext.as_bytes())
      .map_err(|e| anyhow!("Encryption failed: {}", e))?;

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
      Aes256Gcm, Nonce,
      aead::{Aead, KeyInit},
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

struct MitmServer {
  // MITM's own RSA key pair for communicating with clients
  private_key: RsaPrivateKey,
  public_key_spki_b64: String,

  // Target server URL (base URL without query params)
  server_base_url: String,

  // Connection tracking
  connections: Arc<Mutex<HashMap<String, MitmConnection>>>,
}

struct MitmConnection {
  client_crypto: CryptoContext,
  server_crypto: CryptoContext,
}

impl MitmConnection {
  fn new(_conn_id: String) -> Self {
    Self {
      client_crypto: CryptoContext::new(),
      server_crypto: CryptoContext::new(),
    }
  }
}

impl MitmServer {
  fn new(server_url: String) -> Result<Self> {
    let mut rng = ChaCha20Rng::from_rng(&mut rand::rng());

    let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
    let public_key = RsaPublicKey::from(&private_key);

    let public_key_der = public_key.to_public_key_der()?;
    let public_key_spki_b64 = STANDARD.encode(public_key_der.as_bytes());

    Ok(Self {
      private_key,
      public_key_spki_b64,
      server_base_url: server_url,
      connections: Arc::new(Mutex::new(HashMap::new())),
    })
  }

  fn decrypt_client_key_pkcs1v15(&self, encrypted_b64: &str) -> Result<[u8; 32]> {
    let candidates = vec![
      STANDARD.decode(encrypted_b64)?,
      STANDARD.decode(encrypted_b64.replace('-', "+").replace('_', "/"))?,
    ];

    for encrypted_data in candidates {
      if let Ok(decrypted) = self.private_key.decrypt(Pkcs1v15Encrypt, &encrypted_data)
        && decrypted.len() == 32
      {
        let mut key = [0u8; 32];
        key.copy_from_slice(&decrypted);
        return Ok(key);
      }
    }

    Err(anyhow!("RSA v1.5 decrypt failed"))
  }

  fn encrypt_rsa_pkcs1v15(&self, public_key_pem: &str, data: &[u8]) -> Result<String> {
    let public_key = RsaPublicKey::from_public_key_pem(public_key_pem)
      .or_else(|_| RsaPublicKey::from_pkcs1_pem(public_key_pem))?;

    let mut rng = rand::rng();
    let encrypted = public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)?;

    Ok(STANDARD.encode(encrypted))
  }

  async fn connect_to_server(
    &self,
    conn_id: &str,
    query_params: &str,
  ) -> Result<(
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
    [u8; 32],
  )> {
    let server_url_with_params = if query_params.is_empty() {
      self.server_base_url.clone()
    } else {
      format!("{}?{}", self.server_base_url, query_params)
    };

    info!(
      "[{}] Connecting to target server: {}",
      conn_id, server_url_with_params
    );

    let (ws_stream, _) = connect_async(&server_url_with_params).await?;
    let (mut write, mut read) = ws_stream.split();

    let mut _server_public_key_pem = None;
    let aes_key;

    // Handle server handshake
    while let Some(msg) = read.next().await {
      if let Message::Text(text) = msg?
        && let Ok(sdk_msg) = serde_json::from_str::<SdkEncryptedMessage>(&text)
      {
        match sdk_msg {
          SdkEncryptedMessage::ServerKey { key: key_b64 } => {
            info!("[{}] Got ServerKey from target server", conn_id);

            // Convert to PEM format
            let server_pem = spki_b64_to_pem(&key_b64);
            _server_public_key_pem = Some(server_pem.clone());

            // Generate AES key for server communication
            let mut server_aes_key = [0u8; 32];
            rand::rng().fill_bytes(&mut server_aes_key);
            aes_key = server_aes_key;

            // Encrypt AES key with server's public key
            let encrypted_key_b64 = self.encrypt_rsa_pkcs1v15(&server_pem, &server_aes_key)?;

            // Send SdkClientKey to server
            let client_key_msg = SdkEncryptedMessage::sdk_client_key(encrypted_key_b64);

            let json = serde_json::to_string(&client_key_msg)?;
            write.send(Message::Text(json.into())).await?;
            info!("[{}] Sent SdkClientKey to target server", conn_id);

            // Reunite the split streams and return the same connection
            let ws_stream = write
              .reunite(read)
              .map_err(|e| anyhow!("Failed to reunite streams: {}", e))?;
            return Ok((ws_stream, aes_key));
          }
          _ => {
            // Ignore other message types during handshake
            continue;
          }
        }
      }
    }

    Err(anyhow!("Failed to complete server handshake"))
  }

  async fn handle_client(&self, stream: TcpStream, addr: SocketAddr) -> Result<()> {
    let conn_id = format!("{:x}", rand::random::<u32>());
    info!("[{}] New client from {}", conn_id, addr);

    // Extract query parameters during WebSocket handshake
    let query_params = Arc::new(Mutex::new(String::new()));
    let query_params_clone = Arc::clone(&query_params);
    let conn_id_for_callback = conn_id.clone();

    let callback = move |req: &Request, response: Response| {
      if let Some(query_string) = req.uri().query()
        && let Ok(mut params) = query_params_clone.try_lock()
      {
        *params = query_string.to_string();
        info!(
          "[{}] Extracted query params: {}",
          conn_id_for_callback, query_string
        );
      }
      Ok(response)
    };

    let ws_stream = accept_hdr_async(stream, callback).await?;

    let query_params = {
      let params = query_params.lock().await;
      params.clone()
    };

    let (mut client_write, mut client_read) = ws_stream.split();

    // Create connection state
    let mitm_conn = MitmConnection::new(conn_id.clone());
    {
      let mut connections = self.connections.lock().await;
      connections.insert(conn_id.clone(), mitm_conn);
    }

    // Send ServerKey to client (MITM's own key)
    let server_key_msg = SdkEncryptedMessage::server_key(self.public_key_spki_b64.clone());

    let server_key_json = serde_json::to_string(&server_key_msg)?;
    client_write
      .send(Message::Text(server_key_json.into()))
      .await?;
    info!("[{}] Sent MITM ServerKey to client", conn_id);

    // Handle client messages
    while let Some(msg) = client_read.next().await {
      match msg? {
        Message::Text(text) => {
          if let Ok(sdk_msg) = serde_json::from_str::<SdkEncryptedMessage>(&text) {
            match sdk_msg {
              SdkEncryptedMessage::SdkClientKey {
                key: encrypted_key_b64,
              } => {
                info!("[{}] Got SdkClientKey from client", conn_id);

                // Decrypt client's AES key
                match self.decrypt_client_key_pkcs1v15(&encrypted_key_b64) {
                  Ok(client_aes_key) => {
                    info!(
                      "[{}] Client AES key: {}",
                      conn_id,
                      hex::encode(client_aes_key)
                    );

                    // Store client AES key
                    {
                      let mut connections = self.connections.lock().await;
                      if let Some(conn) = connections.get_mut(&conn_id) {
                        conn.client_crypto.set_aes_key(client_aes_key);
                      }
                    }

                    // Now connect to the real server and get its AES key
                    match self.connect_to_server(&conn_id, &query_params).await {
                      Ok((server_ws, server_aes_key)) => {
                        info!(
                          "[{}] Server AES key: {}",
                          conn_id,
                          hex::encode(server_aes_key)
                        );

                        // Store server AES key
                        {
                          let mut connections = self.connections.lock().await;
                          if let Some(conn) = connections.get_mut(&conn_id) {
                            conn.server_crypto.set_aes_key(server_aes_key);
                          }
                        }

                        info!(
                          "[{}] MITM setup complete! Ready to intercept messages",
                          conn_id
                        );

                        // Start a message interception loop
                        return self
                          .start_message_interception(
                            conn_id.clone(),
                            client_write,
                            client_read,
                            server_ws,
                          )
                          .await;
                      }
                      Err(e) => {
                        error!("[{}] Failed to connect to server: {}", conn_id, e);
                      }
                    }
                  }
                  Err(e) => {
                    error!("[{}] Failed to decrypt client key: {}", conn_id, e);
                  }
                }
              }
              SdkEncryptedMessage::SdkData { .. } => {
                // This should be handled in the interception loop after full setup
                warn!("[{}] Got SdkData before full setup", conn_id);
              }
              SdkEncryptedMessage::ServerKey { .. } => {
                info!("[{}] Unexpected ServerKey message from client", conn_id);
              }
            }
          }
        }
        Message::Close(_) => {
          info!("[{}] Client disconnected", conn_id);
          break;
        }
        _ => {}
      }
    }

    // Cleanup
    {
      let mut connections = self.connections.lock().await;
      connections.remove(&conn_id);
    }

    Ok(())
  }

  async fn start_message_interception(
    &self,
    conn_id: String,
    mut client_write: futures_util::stream::SplitSink<
      tokio_tungstenite::WebSocketStream<TcpStream>,
      Message,
    >,
    mut client_read: futures_util::stream::SplitStream<
      tokio_tungstenite::WebSocketStream<TcpStream>,
    >,
    server_ws: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
  ) -> Result<()> {
    // Split the server WebSocket connection
    let (mut server_write, mut server_read) = server_ws.split();

    // Handle bidirectional message forwarding
    info!("[{}] Starting bidirectional message interception", conn_id);

    loop {
      tokio::select! {
          // Handle client → server messages
          client_msg = client_read.next() => {
              match client_msg {
                  Some(Ok(Message::Text(text))) => {
                      if let Ok(sdk_msg) = serde_json::from_str::<SdkEncryptedMessage>(&text) {
                          match sdk_msg {
                              SdkEncryptedMessage::SdkData { data: encrypted_data } => {
                                  // Decrypt message from client
                                  let connections = self.connections.lock().await;
                                  if let Some(conn) = connections.get(&conn_id) {
                                      match conn.client_crypto.decrypt_aes_gcm(&encrypted_data) {
                                          Ok(plaintext) => {
                                              info!("[{}] 📤 CLIENT → SERVER: {}", conn_id, plaintext);

                                              // Re-encrypt for server
                                              match conn.server_crypto.encrypt_aes_gcm(&plaintext) {
                                                  Ok(server_encrypted) => {
                                                      let server_msg = SdkEncryptedMessage::sdk_data(server_encrypted);

                                                      let server_json = serde_json::to_string(&server_msg)?;
                                                      if let Err(e) = server_write.send(Message::Text(server_json.into())).await {
                                                          error!("[{}] Failed to send to server: {}", conn_id, e);
                                                          break;
                                                      }
                                                  }
                                                  Err(e) => error!("[{}] Failed to encrypt for server: {}", conn_id, e),
                                              }
                                          }
                                          Err(e) => error!("[{}] Failed to decrypt client message: {}", conn_id, e),
                                      }
                                  }
                              }
                              SdkEncryptedMessage::SdkClientKey { .. } => {
                                  info!("[{}] Unexpected SdkClientKey during interception", conn_id);
                              }
                              SdkEncryptedMessage::ServerKey { .. } => {
                                  info!("[{}] Unexpected ServerKey from client during interception", conn_id);
                              }
                          }
                      }
                  }
                  Some(Ok(Message::Close(_))) => {
                      info!("[{}] Client disconnected during interception", conn_id);
                      break;
                  }
                  Some(Err(e)) => {
                      error!("[{}] Client read error: {}", conn_id, e);
                      break;
                  }
                  None => {
                      info!("[{}] Client stream ended", conn_id);
                      break;
                  }
                  _ => {}
              }
          }

          // Handle server → client messages
          server_msg = server_read.next() => {
              match server_msg {
                  Some(Ok(Message::Text(text))) => {
                      // info!("[{}] Received from server: {}", conn_id, text);
                      if let Ok(sdk_msg) = serde_json::from_str::<SdkEncryptedMessage>(&text) {
                          match sdk_msg {
                              SdkEncryptedMessage::SdkData { data: encrypted_data } => {
                                  // Decrypt message from server
                                  let connections = self.connections.lock().await;
                                  if let Some(conn) = connections.get(&conn_id) {
                                      match conn.server_crypto.decrypt_aes_gcm(&encrypted_data) {
                                          Ok(plaintext) => {
                                              info!("[{}] 📥 SERVER → CLIENT: {}", conn_id, plaintext);

                                              // Re-encrypt for client
                                              match conn.client_crypto.encrypt_aes_gcm(&plaintext) {
                                                  Ok(client_encrypted) => {
                                                      let client_msg = SdkEncryptedMessage::sdk_data(client_encrypted);

                                                      let client_json = serde_json::to_string(&client_msg)?;
                                                      if let Err(e) = client_write.send(Message::Text(client_json.into())).await {
                                                          error!("[{}] Failed to send to client: {}", conn_id, e);
                                                          break;
                                                      }
                                                  }
                                                  Err(e) => error!("[{}] Failed to encrypt for client: {}", conn_id, e),
                                              }
                                          }
                                          Err(e) => error!("[{}] Failed to decrypt server message: {}", conn_id, e),
                                      }
                                  }
                              }
                              SdkEncryptedMessage::ServerKey { .. } => {
                                  info!("[{}] Unexpected ServerKey from server during interception", conn_id);
                              }
                              SdkEncryptedMessage::SdkClientKey { .. } => {
                                  info!("[{}] Unexpected SdkClientKey from server during interception", conn_id);
                              }
                          }
                      }
                  }
                  Some(Ok(Message::Close(_))) => {
                      info!("[{}] Server disconnected during interception", conn_id);
                      break;
                  }
                  Some(Err(e)) => {
                      error!("[{}] Server read error: {}", conn_id, e);
                      break;
                  }
                  None => {
                      info!("[{}] Server stream ended", conn_id);
                      break;
                  }
                  _ => {}
              }
          }
      }
    }

    Ok(())
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

  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    eprintln!("Usage: {} <listen_port> <target_server_url>", args[0]);
    eprintln!(
      "Example: {} 8080 \"ws://127.0.0.1:15881/v4/feedback\"",
      args[0]
    );
    return Ok(());
  }

  let listen_port = &args[1];
  let server_url = args[2].clone();

  let mitm_server = MitmServer::new(server_url)?;

  info!(
    "\n[MITM] Public key (base64):\n{}",
    mitm_server.public_key_spki_b64
  );

  let addr = format!("127.0.0.1:{}", listen_port);
  let listener = TcpListener::bind(&addr).await?;
  info!("MITM server listening on ws://{}/v4/feedback", addr);
  info!("Clients should connect to this MITM server instead of the real server");

  let mitm_server = Arc::new(mitm_server);

  while let Ok((stream, client_addr)) = listener.accept().await {
    let mitm_server_clone = Arc::clone(&mitm_server);

    tokio::spawn(async move {
      info!("Accepted connection from {}", client_addr);
      if let Err(e) = mitm_server_clone.handle_client(stream, client_addr).await {
        error!("Error handling client {}: {}", client_addr, e);
      }
    });
  }

  Ok(())
}
