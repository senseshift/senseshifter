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
use tokio_tungstenite::tungstenite::{Error as TungsteniteError, Error};
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

                let file_path = p.join(format!("{}.jsonl", file_name));
                let metadata_file_path = p.join(format!("{}.metadata", file_name));

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

        match request.uri().path().split("/").nth(1) {
            #[cfg(feature = "v1")]
            Some("v1") => {
                // unimplemented!("v1 SDK is not yet!")
            },
            #[cfg(feature = "v2")]
            Some("v2") => {
                // unimplemented!("v2 SDK is not yet!")
            },
            #[cfg(feature = "v3")]
            Some("v3") => {
                // unimplemented!("v3 SDK is not yet!")
            }
            #[cfg(feature = "v4")]
            Some("v4") => {
                // unimplemented!("v4 SDK is not yet!")
            }
            _ => {
                warn!("Unsupported path: {}", request.uri().path());
                // return Err(anyhow::anyhow!("Unsupported API"));
            }
        }

        let (mut ws_sender, mut ws_receiver) = stream.split();

        loop {
            tokio::select! {
                result = ws_receiver.next() => {
                    match result {
                        Some(Ok(msg)) => {
                            debug!("Received message: {:?}", msg);

                            if msg.is_text() || msg.is_binary() {
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
                            } else if msg.is_close() {
                                info!("Received close message from peer");
                                break;
                            }

                            // Echo the message back
                            ws_sender.send(msg).await?;
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

        Ok(())
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