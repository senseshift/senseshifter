use std::net::SocketAddr;
use std::time::Duration;
use derivative::Derivative;

/// Configuration for connection management
#[derive(Derivative)]
#[derivative(Debug, Clone, Default)]
pub struct ConnectionManagerConfig {
    /// Buffer size for packet channels
    #[derivative(Default(value = "1000"))]
    pub packet_buffer_size: usize,
    /// Health check interval in seconds
    #[derivative(Default(value = "Duration::from_secs(5)"))]
    pub health_check_interval: Duration,
    /// Maximum concurrent reconnection attempts
    #[derivative(Default(value = "10"))]
    pub max_concurrent_reconnections: usize,
}

impl ConnectionManagerConfig {
    pub fn builder() -> ConnectionManagerConfigBuilder {
        ConnectionManagerConfigBuilder::default()
    }
}

#[derive(Derivative)]
#[derivative(Debug, Clone, Default)]
pub struct ConnectionManagerConfigBuilder {
    #[derivative(Default(value = "None"))]
    packet_buffer_size: Option<usize>,
    #[derivative(Default(value = "None"))]
    health_check_interval: Option<Duration>,
    #[derivative(Default(value = "None"))]
    max_concurrent_reconnections: Option<usize>,
}

impl ConnectionManagerConfigBuilder {
    pub fn packet_buffer_size(mut self, size: usize) -> Self {
        self.packet_buffer_size = Some(size);
        self
    }

    pub fn health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = Some(interval);
        self
    }

    pub fn max_concurrent_reconnections(mut self, max: usize) -> Self {
        self.max_concurrent_reconnections = Some(max);
        self
    }

    pub fn build(self) -> ConnectionManagerConfig {
        let default = ConnectionManagerConfig::default();
        ConnectionManagerConfig {
            packet_buffer_size: self.packet_buffer_size.unwrap_or(default.packet_buffer_size),
            health_check_interval: self.health_check_interval.unwrap_or(default.health_check_interval),
            max_concurrent_reconnections: self.max_concurrent_reconnections.unwrap_or(default.max_concurrent_reconnections),
        }
    }
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
    // Future extensibility - these will be easy to add
    // WebSocket(WebSocketTransportConfig),
    // Grpc(GrpcTransportConfig),
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

/// OSC connection target configuration
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub transport: TransportConfig,
    /// Reconnect interval for failed connections
    #[derivative(Default(value = "Duration::from_secs(5)"))]
    pub reconnect_interval: Duration,
}

impl Target {
    pub fn new(name: String, transport: TransportConfig, reconnect_interval: Duration) -> Self {
        Self { name, transport, reconnect_interval }
    }
    
    pub fn udp(name: String, to: SocketAddr, reconnect_interval: Duration) -> Self {
        Self::new(name, TransportConfig::udp(to), reconnect_interval)
    }
    
    pub fn tcp(name: String, to: SocketAddr, reconnect_interval: Duration) -> Self {
        Self::new(name, TransportConfig::tcp(to), reconnect_interval)
    }

    pub fn builder(name: impl Into<String>) -> TargetBuilder {
        TargetBuilder::new(name.into())
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct TargetBuilder {
    name: String,
    #[derivative(Default(value = "None"))]
    transport: Option<TransportConfig>,
    #[derivative(Default(value = "None"))]
    reconnect_interval: Option<Duration>,
}

impl TargetBuilder {
    fn new(name: String) -> Self {
        Self {
            name,
            transport: None,
            reconnect_interval: None,
        }
    }

    pub fn udp(mut self, to: SocketAddr) -> Self {
        self.transport = Some(TransportConfig::udp(to));
        self
    }

    pub fn tcp(mut self, to: SocketAddr) -> Self {
        self.transport = Some(TransportConfig::tcp(to));
        self
    }

    pub fn reconnect_interval(mut self, interval: Duration) -> Self {
        self.reconnect_interval = Some(interval);
        self
    }

    pub fn build(self) -> Result<Target, &'static str> {
        let transport = self.transport.ok_or("Transport configuration is required")?;
        let reconnect_interval = self.reconnect_interval.unwrap_or(Duration::from_secs(5));
        
        Ok(Target::new(self.name, transport, reconnect_interval))
    }
}

// Re-export for backward compatibility
pub use UdpTransportConfig as OscTransportUdpConfig;
pub use TcpTransportConfig as OscTransportTcpConfig;
pub use TransportConfig as OscTransportConfig;
pub use Target as OscTarget;

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, Ipv4Addr};
    use std::time::Duration;

    #[test]
    fn test_connection_manager_config_defaults() {
        let config = ConnectionManagerConfig::default();
        assert_eq!(config.packet_buffer_size, 1000);
        assert_eq!(config.health_check_interval, Duration::from_secs(5));
        assert_eq!(config.max_concurrent_reconnections, 10);
    }

    #[test]
    fn test_connection_manager_config_builder() {
        let config = ConnectionManagerConfig::builder()
            .packet_buffer_size(500)
            .health_check_interval(Duration::from_secs(10))
            .max_concurrent_reconnections(5)
            .build();
            
        assert_eq!(config.packet_buffer_size, 500);
        assert_eq!(config.health_check_interval, Duration::from_secs(10));
        assert_eq!(config.max_concurrent_reconnections, 5);
    }

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

    #[test]
    fn test_target_builder() {
        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000);
        
        let target = Target::builder("test")
            .udp(addr)
            .reconnect_interval(Duration::from_secs(10))
            .build()
            .unwrap();
            
        assert_eq!(target.name, "test");
        assert_eq!(target.reconnect_interval, Duration::from_secs(10));
    }
}