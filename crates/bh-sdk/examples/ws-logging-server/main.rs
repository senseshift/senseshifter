use futures_util::{SinkExt, StreamExt};
use tracing::*;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{Error, Message, Result},
};
use tungstenite::handshake::server::{Request, Response};
use bh_sdk::v2::{Message as BhV2Message, ResponseMessage as BhV2ResponseMessage};

#[derive(Debug, Clone)]
enum ApiVersion {
    V1,
    V2,
    V3,
    V4,
    Unknown,
}

fn extract_version_from_uri(uri: &str) -> ApiVersion {
    if uri.starts_with("/v1/") {
        ApiVersion::V1
    } else if uri.starts_with("/v2/") {
        ApiVersion::V2
    } else if uri.starts_with("/v3/") {
        ApiVersion::V3
    } else if uri.starts_with("/v4/") {
        ApiVersion::V4
    } else {
        ApiVersion::Unknown
    }
}

async fn handle_v1_message(msg: &str, _ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>) -> Result<()> {
    info!("Handling V1 message: {}", msg);
    // V1 handler placeholder - implement specific V1 logic here
    Ok(())
}

async fn handle_v2_message(msg: &str, ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>, registered_keys: &mut Vec<String>) -> Result<()> {
    let message = match serde_json::from_str::<BhV2Message>(msg) {
        Ok(bh_msg) => bh_msg,
        Err(e) => {
            warn!("Failed to parse BhV2Message: {}", e);
            return Ok(());
        }
    };

    info!("Parsed BhV2Message: {:?}", message);

    match message {
        BhV2Message::Register(registers) => {
            for reg in registers {
                if !registered_keys.contains(&reg.key()) {
                    registered_keys.push(reg.key().to_string());
                }
            }

            let response = BhV2ResponseMessage::RegisteredKeys(registered_keys.clone());
            let response_text = serde_json::to_string(&response).unwrap();
            ws_sender.send(Message::Text(response_text.into())).await?;
        }
        _ => {
            warn!("Unhandled BhV2Message type");
        }
    }
    
    Ok(())
}

async fn handle_v3_message(msg: &str, _ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>) -> Result<()> {
    info!("Handling V3 message: {}", msg);
    // V3 handler placeholder - implement specific V3 logic here
    Ok(())
}

async fn handle_v4_message(msg: &str, _ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>) -> Result<()> {
    info!("Handling V4 message: {}", msg);
    // V4 handler placeholder - implement specific V4 logic here
    Ok(())
}

async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8(_) => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) -> Result<()> {
    use std::sync::{Arc, Mutex};
    
    let api_version = Arc::new(Mutex::new(ApiVersion::Unknown));
    let api_version_clone = api_version.clone();
    
    let callback = move |req: &Request, mut response: Response| {
        info!("Received a new ws handshake");

        let uri_str = req.uri().to_string();
        info!("The request's URI is: {}", uri_str);
        
        let version = extract_version_from_uri(&uri_str);
        info!("Detected API version: {:?}", version);
        
        *api_version_clone.lock().unwrap() = version;
        
        info!("The request's headers are:");
        for (ref header, _value) in req.headers() {
            info!("* {}: {:?}", header, _value);
        }

        let headers = response.headers_mut();
        headers.append("MyCustomHeader", ":)".parse().unwrap());

        Ok(response)
    };

    let ws_stream = accept_hdr_async(stream, callback).await.expect("Failed to accept");
    let detected_version = api_version.lock().unwrap().clone();
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // create file to log messages
    let log_file_path = format!("data/ws_log_{}_{}_{}.jsonl", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(), peer.ip(), peer.port());
    let mut log_file = tokio::fs::File::create(&log_file_path).await?;
    info!("Logging messages to {}", log_file_path);

    let mut registered_keys: Vec<String> = Vec::new();

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() || msg.is_binary() {
                            info!("Received a message from {} (API version: {:?})", peer, detected_version);

                            tokio::io::AsyncWriteExt::write_all(&mut log_file, msg.to_text()?.as_bytes()).await?;
                            tokio::io::AsyncWriteExt::write_all(&mut log_file, b"\n").await?;

                            let msg_text = msg.to_text()?;
                            
                            // Route message to appropriate handler based on detected API version
                            match detected_version {
                                ApiVersion::V1 => {
                                    if let Err(e) = handle_v1_message(msg_text, &mut ws_sender).await {
                                        error!("Error handling V1 message: {}", e);
                                    }
                                }
                                ApiVersion::V2 => {
                                    if let Err(e) = handle_v2_message(msg_text, &mut ws_sender, &mut registered_keys).await {
                                        error!("Error handling V2 message: {}", e);
                                    }
                                }
                                ApiVersion::V3 => {
                                    if let Err(e) = handle_v3_message(msg_text, &mut ws_sender).await {
                                        error!("Error handling V3 message: {}", e);
                                    }
                                }
                                ApiVersion::V4 => {
                                    if let Err(e) = handle_v4_message(msg_text, &mut ws_sender).await {
                                        error!("Error handling V4 message: {}", e);
                                    }
                                }
                                ApiVersion::Unknown => {
                                    error!("Received message for unknown API version");
                                }
                            }
                        } else if msg.is_close() {
                            info!("Received close message from {}", peer);
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }

    info!("Connection to {} closed.", peer);

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:15881";
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    loop {
        tokio::select! {
            Ok((stream, addr)) = listener.accept() => {
                let peer = stream.peer_addr().expect("connected streams should have a peer address");
                info!("New WebSocket connection: {}. Peer address: {}", addr, peer);

                tokio::spawn(accept_connection(peer, stream));
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down server.");
                break;
            }
        }
    }
}
