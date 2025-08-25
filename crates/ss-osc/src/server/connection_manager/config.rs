use std::net::SocketAddr;
use std::time::Duration;
use derivative::Derivative;
use crate::server::config::RouterForwardTargetConfig;

/// Configuration for connection management
#[derive(Derivative)]
#[derivative(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConnectionManagerConfig {
    /// Buffer size for packet channels
    #[derivative(Default(value = "1000"))]
    pub packet_buffer_size: usize,

    /// Maximum concurrent reconnection attempts
    #[derivative(Default(value = "10"))]
    pub max_concurrent_reconnections: usize,
}

// ============================================================================
// Transport Configurations
// ============================================================================

/// UDP transport configuration
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
pub struct UdpTransportConfig {
    /// Remote target address
    pub to: SocketAddr,
}

impl UdpTransportConfig {
    pub fn new(to: SocketAddr) -> Self {
        Self { to }
    }
}

/// TCP transport configuration  
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
pub struct TcpTransportConfig {
    /// Remote target address
    pub to: SocketAddr,
}

impl TcpTransportConfig {
    pub fn new(to: SocketAddr) -> Self {
        Self { to }
    }
}

/// Transport configuration enum supporting different OSC transport types
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
pub enum TransportConfig {
    Udp(UdpTransportConfig),
    Tcp(TcpTransportConfig),
    // todo: add WebSocket, etc.
}

impl TransportConfig {
    /// Get the remote address for connection (for UDP/TCP)
    /// For future transports, this might return a different identifier
    pub fn remote_address(&self) -> String {
        match self {
            TransportConfig::Udp(config) => config.to.to_string(),
            TransportConfig::Tcp(config) => config.to.to_string(),
        }
    }
    
    /// Get transport type as string for logging
    pub fn transport_type(&self) -> &'static str {
        match self {
            TransportConfig::Udp(_) => "UDP",
            TransportConfig::Tcp(_) => "TCP",
        }
    }
    
    /// Create UDP transport config
    pub fn udp(to: SocketAddr) -> Self {
        Self::Udp(UdpTransportConfig::new(to))
    }
    
    /// Create TCP transport config
    pub fn tcp(to: SocketAddr) -> Self {
        Self::Tcp(TcpTransportConfig::new(to))
    }
}

impl From<RouterForwardTargetConfig> for TransportConfig {
    fn from(config: RouterForwardTargetConfig) -> Self {
        match config {
            RouterForwardTargetConfig::Udp(udp_config) => {
                TransportConfig::Udp(UdpTransportConfig::new(udp_config.to))
            }
            RouterForwardTargetConfig::Tcp(tcp_config) => {
                TransportConfig::Tcp(TcpTransportConfig::new(tcp_config.to))
            }
        }
    }
}

/// OSC connection target configuration
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub transport: TransportConfig,
}

impl Target {
    pub fn new(name: String, transport: TransportConfig) -> Self {
        Self { name, transport }
    }
    
    pub fn udp(name: String, to: SocketAddr) -> Self {
        Self::new(name, TransportConfig::udp(to))
    }
    
    pub fn tcp(name: String, to: SocketAddr) -> Self {
        Self::new(name, TransportConfig::tcp(to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, Ipv4Addr};
    use std::time::Duration;

    #[test]
    fn test_transport_config() {
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000);
        
        let udp_config = TransportConfig::udp(addr);
        assert_eq!(udp_config.remote_address(), addr.to_string());
        assert_eq!(udp_config.transport_type(), "UDP");
        
        let tcp_config = TransportConfig::tcp(addr);
        assert_eq!(tcp_config.remote_address(), addr.to_string());
        assert_eq!(tcp_config.transport_type(), "TCP");
    }
}