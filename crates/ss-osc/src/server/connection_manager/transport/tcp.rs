use derivative::Derivative;
use rosc::OscPacket;
use tokio_util::sync::CancellationToken;
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use rosc::encoder;
use tracing::{debug, error};

use crate::server::connection_manager::{TcpTransportConfig, ConnectionError};

/// TCP transport handler
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct TcpHandler {
    config: TcpTransportConfig,
}

impl TcpHandler {
    pub fn new(config: TcpTransportConfig) -> Self {
        Self { config }
    }
}

impl TcpHandler {
    pub async fn start(
        &self,
        mut packet_rx: mpsc::Receiver<OscPacket>,
        cancellation_token: CancellationToken,
    ) -> Result<(), ConnectionError> {
        let mut stream = TcpStream::connect(self.config.to).await
            .map_err(|e| ConnectionError::Transport(format!("Failed to connect TCP stream to {}: {}", self.config.to, e)))?;

        debug!("TCP sender task started for {}", self.config.to);

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("TCP sender task cancelled for {}", self.config.to);
                    break;
                }
                Some(packet) = packet_rx.recv() => {
                    match encoder::encode_tcp(&packet) {
                        Ok(encoded) => {
                            if let Err(e) = stream.write_all(&encoded).await {
                                error!("Failed to send TCP packet to {}: {}", self.config.to, e);
                                return Err(ConnectionError::Send(format!("TCP send failed: {}", e)));
                            }
                            debug!("Sent TCP packet to {}", self.config.to);
                        }
                        Err(e) => {
                            error!("Failed to encode TCP packet for {}: {}", self.config.to, e);
                            return Err(ConnectionError::Encoding(format!("Failed to encode packet: {}", e)));
                        }
                    }
                }
                else => break,
            }
        }

        debug!("TCP sender task ended for {}", self.config.to);
        Ok(())
    }

    pub fn transport_type(&self) -> &'static str {
        "TCP"
    }

    pub fn display_name(&self) -> String {
        format!("TCP(-> {})", self.config.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, Ipv4Addr};

    #[test]
    fn test_tcp_handler_display_name() {
        let config = TcpTransportConfig::new(
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000)
        );
        let handler = TcpHandler::new(config);
        assert_eq!(handler.display_name(), "TCP(-> 127.0.0.1:9000)");
        assert_eq!(handler.transport_type(), "TCP");
    }
}