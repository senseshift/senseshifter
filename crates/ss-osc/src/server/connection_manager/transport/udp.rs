use derivative::Derivative;
use rosc::OscPacket;
use tokio_util::sync::CancellationToken;
use tokio::sync::{mpsc, broadcast};
use tokio::net::UdpSocket;
use rosc::encoder;
use tracing::{debug, error, info};

use crate::server::connection_manager::{ConnectionError, ConnectionEvent};
use super::super::config::{Target, TransportConfig};

/// UDP transport handler
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct UdpHandler;

impl UdpHandler {
    pub fn new() -> Self {
        Self
    }
}

impl UdpHandler {
    pub async fn start(
        &self,
        mut packet_rx: mpsc::Receiver<OscPacket>,
        cancellation_token: CancellationToken,
        target: Target,
        event_sender: broadcast::Sender<ConnectionEvent>,
    ) -> Result<(), ConnectionError> {
        let udp_config = match &target.transport {
            TransportConfig::Udp(config) => config,
            _ => return Err(ConnectionError::Transport("UDP handler called with non-UDP target".to_string())),
        };
        
        let bind_addr = "0.0.0.0:0"; // Let OS choose local port
        debug!("Binding UDP socket to {} for target {}", bind_addr, udp_config.to);

        let socket = UdpSocket::bind(bind_addr).await
            .map_err(|e| ConnectionError::Transport(format!("Failed to bind UDP socket to {}: {}", bind_addr, e)))?;

        socket.connect(udp_config.to).await
            .map_err(|e| ConnectionError::Transport(format!("Failed to connect UDP socket to {}: {}", udp_config.to, e)))?;

        // UDP is connectionless - emit Connected immediately
        info!("UDP target {} ready to send packets to {}", target.name, udp_config.to);
        let _ = event_sender.send(ConnectionEvent::Connected { target: target.clone() });
        
        debug!("UDP sender task started for {}", udp_config.to);

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("UDP sender task cancelled for {}", udp_config.to);
                    break;
                }
                Some(packet) = packet_rx.recv() => {
                    match encoder::encode(&packet) {
                        Ok(encoded) => {
                            if let Err(e) = socket.send(&encoded).await {
                                error!("Failed to send UDP packet to {}: {}", udp_config.to, e);
                                return Err(ConnectionError::Send(format!("UDP send failed: {}", e)));
                            }
                            debug!("Sent UDP packet to {}", udp_config.to);
                        }
                        Err(e) => {
                            error!("Failed to encode packet for {}: {}", udp_config.to, e);
                            return Err(ConnectionError::Encoding(format!("Failed to encode packet: {}", e)));
                        }
                    }
                }
                else => break,
            }
        }

        debug!("UDP sender task ended for {}", udp_config.to);
        Ok(())
    }

    pub fn transport_type(&self) -> &'static str {
        "UDP"
    }

    pub fn display_name(&self, target: &Target) -> String {
        match &target.transport {
            TransportConfig::Udp(config) => format!("UDP(-> {})", config.to),
            _ => "UDP(invalid config)".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, Ipv4Addr};

    #[test]
    fn test_udp_handler_display_name() {
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000);
        let target = Target::udp("test".to_string(), addr);
        let handler = UdpHandler::new();
        assert_eq!(handler.display_name(&target), "UDP(-> 127.0.0.1:9000)");
        assert_eq!(handler.transport_type(), "UDP");
    }
}