pub mod udp;
pub mod tcp;

use rosc::OscPacket;
use tokio_util::sync::CancellationToken;
use tokio::sync::mpsc;

use super::config::TransportConfig;
use super::error::ConnectionError;

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
    ) -> Result<(), ConnectionError> {
        match self {
            TransportHandler::Udp(handler) => handler.start(packet_rx, cancellation_token).await,
            TransportHandler::Tcp(handler) => handler.start(packet_rx, cancellation_token).await,
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
    pub fn display_name(&self) -> String {
        match self {
            TransportHandler::Udp(handler) => handler.display_name(),
            TransportHandler::Tcp(handler) => handler.display_name(),
        }
    }
}

/// Factory for creating transport handlers
pub fn create_transport_handler(config: &TransportConfig) -> TransportHandler {
    match config {
        TransportConfig::Udp(udp_config) => TransportHandler::Udp(UdpHandler::new(udp_config.clone())),
        TransportConfig::Tcp(tcp_config) => TransportHandler::Tcp(TcpHandler::new(tcp_config.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, Ipv4Addr};

    #[test]
    fn test_create_transport_handler() {
        let udp_config = TransportConfig::udp(
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000)
        );
        let handler = create_transport_handler(&udp_config);
        assert_eq!(handler.transport_type(), "UDP");

        let tcp_config = TransportConfig::tcp(
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000)
        );
        let handler = create_transport_handler(&tcp_config);
        assert_eq!(handler.transport_type(), "TCP");
    }
}