use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use crate::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_hdr_async, WebSocketStream};
use tokio_util::sync::CancellationToken;
use tracing::{*, Instrument};
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

pub(crate) struct BhServerTask {
    listen: Vec<SocketAddr>,
    cancellation_token: CancellationToken,
    sniff_into: Option<PathBuf>,
}

impl BhServerTask {
    pub(crate) fn new(
        listen: Vec<SocketAddr>,
        cancellation_token: CancellationToken,
        sniff_into: Option<PathBuf>,
    ) -> Self {
        Self {
            listen,
            cancellation_token,
            sniff_into,
        }
    }

    #[tracing::instrument(
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
                            info!("New connection from {}", addr);

                            let conn_cancellation_token = self.cancellation_token.child_token();

                            let connection_task = BhServerConnectionTask {
                                peer: addr,
                                cancellation_token: conn_cancellation_token,
                                sniff_into: self.sniff_into.clone(),
                            };

                            tokio::spawn(
                                async move {
                                    match connection_task.handle_client(stream).await {
                                        Ok(_) => {
                                            info!("Connection to {} handled successfully.", addr);
                                        }
                                        Err(e) => {
                                            error!("Error handling connection to {}: {}", addr, e);
                                        }
                                    }
                                }.instrument(info_span!("Connection Task", peer = %addr))
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
    peer: SocketAddr,
    cancellation_token: CancellationToken,

    /// If set, the connection will be sniffed into the given directory.
    sniff_into: Option<PathBuf>,
}

impl BhServerConnectionTask {

    #[tracing::instrument(
        skip(self, stream),
    )]
    async fn handle_client(&self, stream: TcpStream) -> Result<()> {
        let request = Arc::new(Mutex::new(None));

        let request_clone = request.clone();
        let callback = move |req: &Request, response: Response| {
            *request_clone.lock().unwrap() = Some(req.clone());

            Ok(response)
        };

        let ws_stream = accept_hdr_async(stream, callback).await?;

        // wait for the request to be filled by the callback
        let request = loop {
            trace!("Waiting for request...");

            if let Some(req) = request.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock request: {:?}", e))?
                .as_ref() {
                break req.clone();
            }

            tokio::task::yield_now().await;
        };

        debug!("Received request: {:?}", request);

        self.handle_connection(&request, ws_stream).await
    }

    #[tracing::instrument(
        skip(self, request, stream),
        fields(
            uri = %request.uri()
        )
    )]
    async fn handle_connection(
        &self,
        request: &Request,
        stream: WebSocketStream<TcpStream>,
    ) -> Result<()> {
        let sniff_files = self.sniff_into.clone()
            .map(|p| {
                let timestamp: String = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string();

                let file_name = format!("{}_{}_{}", timestamp, self.peer.ip(), self.peer.port());

                let prefix = timestamp.to_string().get(0..5)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let file_path = p.join(prefix.clone()).join(format!("{}.jsonl", file_name));
                let metadata_file_path = p.join(prefix).join(format!("{}.metadata", file_name));

                (file_path, metadata_file_path)
            });

        let (sniff_file, sniff_metadata_file): (Option<File>, Option<File>) = match sniff_files.as_ref() {
            Some((ref file_path, ref metadata_file_path)) => (open_or_none(file_path), open_or_none(metadata_file_path)),
            None => (None, None),
        };

        if let Some(mut sniff_metadata_file) = sniff_metadata_file.as_ref() {
            let request_string = format!("{:#?}", request);

            match sniff_metadata_file.write_all([request_string.as_bytes(), b"\n"].concat().as_slice()) {
                Ok(_) => {
                    info!("Wrote request metadata to {:?}", sniff_metadata_file);
                }
                Err(e) => {
                    error!("Failed to write request metadata to {:?}: {}", sniff_metadata_file, e);
                }
            }
        }

        let path = request.uri().path();
        let version = extract_api_version_from_path(path);
        
        match version {
            #[cfg(feature = "v1")]
            Some(1) => {
                info!("Handling V1 API request");
                // TODO: Route to V1 handler
            },
            #[cfg(feature = "v2")]
            Some(2) => {
                info!("Handling V2 API request");
                // TODO: Route to V2 handler  
            },
            #[cfg(feature = "v3")]
            Some(3) => {
                info!("Handling V3 API request");
                // TODO: Route to V3 handler
            }
            #[cfg(feature = "v4")]
            Some(4) => {
                info!("Handling V4 API request");
                // TODO: Route to V4 handler
            }
            Some(version) => {
                warn!("Unsupported API version: v{} for path: {}", version, path);
            }
            None => {
                warn!("No API version detected in path: {}", path);
            }
        }

        let (mut ws_sender, mut ws_receiver) = stream.split();

        loop {
            tokio::select! {
                result = ws_receiver.next() => {
                    match result {
                        Some(Ok(msg)) => {
                            debug!("Received message: {:?}", msg);

                            match msg {
                                Message::Text(_) | Message::Binary(_) => {
                                    if let Some(mut sniff_file) = sniff_file.as_ref() {
                                        match sniff_file.write_all([msg.to_text()?.as_bytes(), b"\n"].concat().as_slice()) {
                                            Ok(_) => {
                                                trace!("Wrote message to {:?}", sniff_file);
                                            }
                                            Err(e) => {
                                                error!("Failed to write message to {:?}: {}", sniff_file, e);
                                            }
                                        }
                                    }

                                    // todo: handle & route message
                                }
                                Message::Close(close_frame) => {
                                    info!("Received close message from peer: {:?}", close_frame);
                                    // Respond with a close frame to complete the close handshake
                                    let _ = ws_sender.close().await;
                                    break;
                                }
                                Message::Ping(data) => {
                                    debug!("Received ping from peer");
                                    // Respond with pong
                                    if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                                        error!("Failed to send pong: {}", e);
                                    }
                                }
                                Message::Pong(_) => {
                                    debug!("Received pong from peer");
                                    // Just acknowledge, no action needed
                                }
                                Message::Frame(_) => {
                                    warn!("Received raw frame, this shouldn't happen in high-level API");
                                }
                            }
                        }
                        Some(Err(Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8(_))) => {
                            info!("Connection closed by peer: {}", self.peer);
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            info!("WebSocket stream ended for peer: {}", self.peer);
                            break;
                        }
                    }
                }
                _ = self.cancellation_token.cancelled() => {
                    info!("Cancellation token triggered, closing connection");
                    ws_sender.close().await?;
                    break;
                }
            }
        }

        if let Some(mut metadata_file) = sniff_metadata_file.as_ref() {
            metadata_file.sync_all()?;

            // write close time
            let close_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            match metadata_file.write_all(format!("close_time: {}\n", close_time).as_bytes()) {
                Ok(_) => {
                    trace!("Wrote close time to {:?}", metadata_file);
                }
                Err(e) => {
                    error!("Failed to write close time to {:?}: {}", metadata_file, e);
                }
            }
        }

        Ok(())
    }
}

/// Extract API version from path like "/v2/feedbacks" -> Some(2)
fn extract_api_version_from_path(path: &str) -> Option<u8> {
    // Match patterns like "/v1/...", "/v2/...", "/v3/...", "/v4/..."
    let parts: Vec<&str> = path.split('/').collect();
    
    // Path should be like ["", "v2", "feedbacks", ...] after splitting "/"
    if parts.len() >= 2 {
        let version_part = parts[1];
        
        // Check if it starts with "v" followed by a number
        if version_part.starts_with('v') && version_part.len() >= 2 {
            let version_str = &version_part[1..]; // Remove the "v" prefix
            
            // Try to parse the version number
            match version_str.parse::<u8>() {
                Ok(version) if (1..=255).contains(&version) => Some(version),
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn ensure_parent(p: &Path) -> bool {
    if let Some(parent) = p.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("Failed to create directory {:?}: {}", parent, e);
            return false;
        }
    }
    true
}

fn open_or_none(p: &Path) -> Option<File> {
    if !ensure_parent(p) {
        return None;
    }
    match OpenOptions::new()
        .write(true)
        .create(true)   // create if missing; don’t truncate existing content
        .open(p)
    {
        Ok(f) => Some(f),
        Err(e) => {
            error!("Failed to open/create file {:?}: {}", p, e);
            None
        }
    }
}