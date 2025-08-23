use derivative::Derivative;
use rosc::OscPacket;
use tokio_util::sync::CancellationToken;
use tokio::sync::mpsc;
use tokio::net::UdpSocket;
use rosc::encoder;
use tracing::{debug, error};

use crate::server::connection_manager::{UdpTransportConfig, ConnectionError};

/// UDP transport handler
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct UdpHandler {
    config: UdpTransportConfig,
}

impl UdpHandler {
    pub fn new(config: UdpTransportConfig) -> Self {
        Self { config }
    }
}

impl UdpHandler {
    pub async fn start(
        &self,
        mut packet_rx: mpsc::Receiver<OscPacket>,
        cancellation_token: CancellationToken,
    ) -> Result<(), ConnectionError> {
        let bind_addr = "0.0.0.0:0"; // Let OS choose local port
        debug!("Binding UDP socket to {} for target {}", bind_addr, self.config.to);

        let socket = UdpSocket::bind(bind_addr).await
            .map_err(|e| ConnectionError::Transport(format!("Failed to bind UDP socket to {}: {}", bind_addr, e)))?;

        socket.connect(self.config.to).await
            .map_err(|e| ConnectionError::Transport(format!("Failed to connect UDP socket to {}: {}", self.config.to, e)))?;

        debug!("UDP sender task started for {}", self.config.to);

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("UDP sender task cancelled for {}", self.config.to);
                    break;
                }
                Some(packet) = packet_rx.recv() => {
                    match encoder::encode(&packet) {
                        Ok(encoded) => {
                            if let Err(e) = socket.send(&encoded).await {
                                error!("Failed to send UDP packet to {}: {}", self.config.to, e);
                                return Err(ConnectionError::Send(format!("UDP send failed: {}", e)));
                            }
                            debug!("Sent UDP packet to {}", self.config.to);
                        }
                        Err(e) => {
                            error!("Failed to encode packet for {}: {}", self.config.to, e);
                            return Err(ConnectionError::Encoding(format!("Failed to encode packet: {}", e)));
                        }
                    }
                }
                else => break,
            }
        }

        debug!("UDP sender task ended for {}", self.config.to);
        Ok(())
    }

    pub fn transport_type(&self) -> &'static str {
        "UDP"
    }

    pub fn display_name(&self) -> String {
        format!("UDP(-> {})", self.config.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, Ipv4Addr};

    #[test]
    fn test_udp_handler_display_name() {
        let config = UdpTransportConfig::new(
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000)
        );
        let handler = UdpHandler::new(config);
        assert_eq!(handler.display_name(), "UDP(-> 127.0.0.1:9000)");
        assert_eq!(handler.transport_type(), "UDP");
    }
}