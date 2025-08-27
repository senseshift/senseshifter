use anyhow::Context;
use futures_util::StreamExt;
use crate::Result;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_hdr_async;
use tokio_util::sync::CancellationToken;
use tracing::error;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

pub(crate) struct BhServerTask {
    listen: Vec<std::net::SocketAddr>,
    cancellation_token: CancellationToken,
}

impl BhServerTask {
    pub(crate) fn new(
        listen: Vec<std::net::SocketAddr>,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            listen,
            cancellation_token,
        }
    }

    #[tracing::instrument(
        name = "bH Server Task",
        skip(self),
        fields(
            listen = ?self.listen,
        )
    )]
    pub(crate) async fn run(&mut self) -> Result<()> {
        if self.listen.is_empty() {
            return Err(anyhow::anyhow!("At least one listen address must be provided"));
        }

        let listener = TcpListener::bind(&self.listen[..]).await
            .with_context(|| format!("Failed to bind to addresses: {:?}", self.listen))?;

        tracing::info!("bH Server listening on {:?}", self.listen);

        loop {
            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("Cancellation token triggered, shutting down bH server task.");
                    break;
                }
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            let peer = match stream.peer_addr() {
                                Ok(p) => p,
                                Err(e) => {
                                    error!("Failed to get peer address: {}", e);
                                    continue;
                                }
                            };

                            tracing::info!("New connection from {}::{}", addr, peer);

                            let conn_cancellation_token = self.cancellation_token.child_token();

                            let connection_task = BhServerConnectionTask {
                                peer,
                                cancellation_token: conn_cancellation_token,
                            };

                            tokio::spawn(
                                async move {
                                    match connection_task.accept_connection(stream).await {
                                        Ok(_) => {
                                            tracing::info!("Connection to {} handled successfully.", peer);
                                        }
                                        Err(e) => {
                                            error!("Error handling connection to {}: {}", peer, e);
                                        }
                                    }
                                }
                            );
                        }
                        Err(e) => {
                            error!("Error accepting connection: {}", e);
                            // Continue accepting other connections instead of exiting
                            continue;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

struct BhServerConnectionTask {
    peer: std::net::SocketAddr,
    cancellation_token: CancellationToken,
}

impl BhServerConnectionTask {
    async fn accept_connection(self, stream: tokio::net::TcpStream) -> Result<()> {
        match self.handle_connection(stream).await {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                match e {
                    TungsteniteError::ConnectionClosed | TungsteniteError::Protocol(_) | TungsteniteError::Utf8(_) => {
                        tracing::info!("Connection to {} closed: {}", self.peer, e);
                        Ok(())
                    }
                    err => {
                        error!("Error in connection to {}: {}", self.peer, err);
                        Err(anyhow::anyhow!(err))
                    }
                }
            }
        }
    }

    #[tracing::instrument(
        name = "Handle Connection",
        skip(self, stream),
        fields(
            peer = %self.peer,
        )
    )]
    async fn handle_connection(&self, stream: tokio::net::TcpStream) -> std::result::Result<(), TungsteniteError> {
        let callback = move |_request: &Request, response: Response| {
            Ok(response)
        };

        let ws_stream = accept_hdr_async(stream, callback).await?;
        let (_ws_sender, mut ws_receiver) = ws_stream.split();

        loop {
            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("Cancellation token triggered, closing connection to {}.", self.peer);
                    break;
                }
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(message)) => {
                            tracing::info!("Received message from {}: {:?}", self.peer, message);
                            // Handle the message here
                        }
                        Some(Err(e)) => {
                            error!("Error receiving message from {}: {}", self.peer, e);
                            return Err(e);
                        }
                        None => {
                            tracing::info!("Connection to {} closed by peer.", self.peer);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}