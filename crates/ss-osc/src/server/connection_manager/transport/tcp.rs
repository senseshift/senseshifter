use derivative::Derivative;
use rosc::OscPacket;
use tokio_util::sync::CancellationToken;
use tokio::sync::{mpsc, broadcast};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use rosc::encoder;
use tracing::{debug, error, info};

use crate::server::connection_manager::{ConnectionError, ConnectionEvent};
use super::super::config::{Target, TransportConfig};

/// TCP transport handler
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct TcpHandler;

impl TcpHandler {
    pub fn new() -> Self {
        Self
    }
}

impl TcpHandler {
    pub async fn start(
        &self,
        mut packet_rx: mpsc::Receiver<OscPacket>,
        cancellation_token: CancellationToken,
        target: Target,
        event_sender: broadcast::Sender<ConnectionEvent>,
    ) -> Result<(), ConnectionError> {
        let tcp_config = match &target.transport {
            TransportConfig::Tcp(config) => config,
            _ => return Err(ConnectionError::Transport("TCP handler called with non-TCP target".to_string())),
        };
        
        let mut stream = TcpStream::connect(tcp_config.to).await
            .map_err(|e| ConnectionError::Transport(format!("Failed to connect TCP stream to {}: {}", tcp_config.to, e)))?;

        // TCP connection established - emit Connected immediately
        info!("TCP target {} connection established to {}", target.name, tcp_config.to);
        let _ = event_sender.send(ConnectionEvent::Connected { target: target.clone() });

        debug!("TCP sender task started for {}", tcp_config.to);

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("TCP sender task cancelled for {}", tcp_config.to);
                    break;
                }
                Some(packet) = packet_rx.recv() => {
                    match encoder::encode_tcp(&packet) {
                        Ok(encoded) => {
                            if let Err(e) = stream.write_all(&encoded).await {
                                error!("Failed to send TCP packet to {}: {}", tcp_config.to, e);
                                return Err(ConnectionError::Send(format!("TCP send failed: {}", e)));
                            }
                            debug!("Sent TCP packet to {}", tcp_config.to);
                        }
                        Err(e) => {
                            error!("Failed to encode TCP packet for {}: {}", tcp_config.to, e);
                            return Err(ConnectionError::Encoding(format!("Failed to encode packet: {}", e)));
                        }
                    }
                }
                else => break,
            }
        }

        debug!("TCP sender task ended for {}", tcp_config.to);
        Ok(())
    }

    pub fn transport_type(&self) -> &'static str {
        "TCP"
    }

    pub fn display_name(&self, target: &Target) -> String {
        match &target.transport {
            TransportConfig::Tcp(config) => format!("TCP(-> {})", config.to),
            _ => "TCP(invalid config)".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, Ipv4Addr};

    #[test]
    fn test_tcp_handler_display_name() {
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000);
        let target = Target::tcp("test".to_string(), addr);
        let handler = TcpHandler::new();
        assert_eq!(handler.display_name(&target), "TCP(-> 127.0.0.1:9000)");
        assert_eq!(handler.transport_type(), "TCP");
    }
}