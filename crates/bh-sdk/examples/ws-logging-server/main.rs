use futures_util::{SinkExt, StreamExt};
use tracing::*;
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};

use bh_sdk::v2::{Message as BhMessage, ResponseMessage, SubmitMessage as BhSubmitMessage};

async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8(_) => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
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
                            info!("Received a message from {}", peer);
                            // Try to parse the message as a BhMessage

                            tokio::io::AsyncWriteExt::write_all(&mut log_file, msg.to_text()?.as_bytes()).await?;
                            tokio::io::AsyncWriteExt::write_all(&mut log_file, b"\n").await?;

                            let message = match serde_json::from_str::<BhMessage>(msg.to_text()?) {
                                Ok(bh_msg) => bh_msg,
                                Err(e) => {
                                    warn!("Failed to parse BhMessage: {}", e);
                                    continue;
                                }
                            };

                            info!("Parsed BhMessage: {:?}", message);

                            match message {
                                BhMessage::Register(registers) => {
                                    for reg in registers {
                                        if !registered_keys.contains(&reg.key()) {
                                            registered_keys.push(reg.key().to_string());
                                        }
                                    }

                                    let response = ResponseMessage::RegisteredKeys(registered_keys.clone());

                                    let response_text = serde_json::to_string(&response).unwrap();
                                    ws_sender.send(Message::Text(response_text.into())).await?;
                                }
                                _ => {
                                    warn!("Unhandled BhMessage type");
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
