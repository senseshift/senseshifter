use futures_util::{SinkExt, StreamExt};
use tracing::*;
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};

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
    let (_ws_sender, mut ws_receiver) = ws_stream.split();

    // Echo incoming WebSocket messages and send a message periodically every second.

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() ||msg.is_binary() {
                            info!("Received a message from {}: {}", peer, msg.to_text()?);
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
