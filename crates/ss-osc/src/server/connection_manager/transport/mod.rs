pub mod udp;
pub mod tcp;

use rosc::OscPacket;
use tokio_util::sync::CancellationToken;
use tokio::sync::{mpsc, broadcast};

use super::config::{TransportConfig, Target};
use super::error::ConnectionError;
use super::ConnectionEvent;

pub use udp::UdpHandler;
pub use tcp::TcpHandler;

/// Enum for transport handlers that can be cloned
#[derive(Debug, Clone)]
pub enum TransportHandler {
    Udp(UdpHandler),
    Tcp(TcpHandler),
}

impl TransportHandler {
    /// Start the transport sender task
    pub async fn start(
        &self,
        packet_rx: mpsc::Receiver<OscPacket>,
        cancellation_token: CancellationToken,
        target: Target,
        event_sender: broadcast::Sender<ConnectionEvent>,
    ) -> Result<(), ConnectionError> {
        match self {
            TransportHandler::Udp(handler) => handler.start(packet_rx, cancellation_token, target, event_sender).await,
            TransportHandler::Tcp(handler) => handler.start(packet_rx, cancellation_token, target, event_sender).await,
        }
    }

    /// Get the transport type name for logging
    pub fn transport_type(&self) -> &'static str {
        match self {
            TransportHandler::Udp(handler) => handler.transport_type(),
            TransportHandler::Tcp(handler) => handler.transport_type(),
        }
    }

    /// Get a display name for this transport instance
    pub fn display_name(&self, target: &Target) -> String {
        match self {
            TransportHandler::Udp(handler) => handler.display_name(target),
            TransportHandler::Tcp(handler) => handler.display_name(target),
        }
    }
}

/// Factory for creating transport handlers
pub fn create_transport_handler(config: &TransportConfig) -> TransportHandler {
    match config {
        TransportConfig::Udp(_) => TransportHandler::Udp(UdpHandler::new()),
        TransportConfig::Tcp(_) => TransportHandler::Tcp(TcpHandler::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, Ipv4Addr};

    #[test]
    fn test_create_transport_handler() {
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000);
        
        let udp_config = TransportConfig::udp(addr);
        let handler = create_transport_handler(&udp_config);
        assert_eq!(handler.transport_type(), "UDP");

        let tcp_config = TransportConfig::tcp(addr);
        let handler = create_transport_handler(&tcp_config);
        assert_eq!(handler.transport_type(), "TCP");
    }
}