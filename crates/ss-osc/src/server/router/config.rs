use std::net::SocketAddr;
use derivative::Derivative;
use getset::Getters;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct UdpRouterForwardTargetConfig {
    /// Remote target address
    pub to: SocketAddr,
}

impl UdpRouterForwardTargetConfig {
    pub fn new(to: SocketAddr) -> Self {
        Self { to }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TcpRouterForwardTargetConfig {
    /// Remote target address
    pub to: SocketAddr,
}

impl TcpRouterForwardTargetConfig {
    pub fn new(to: SocketAddr) -> Self {
        Self { to }
    }
}

/// Transport configuration enum supporting different OSC transport types
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "lowercase"))]
pub enum RouterForwardTargetConfig {
    #[cfg_attr(feature = "serde", serde(rename = "udp"))]
    Udp(UdpRouterForwardTargetConfig),

    #[cfg_attr(feature = "serde", serde(rename = "tcp"))]
    Tcp(TcpRouterForwardTargetConfig),

    // todo: WebSocket, Unix sockets, etc.
}

impl RouterForwardTargetConfig {
    /// Get the remote address for connection (for UDP/TCP)
    pub fn remote_address(&self) -> SocketAddr {
        match self {
            RouterForwardTargetConfig::Udp(config) => config.to,
            RouterForwardTargetConfig::Tcp(config) => config.to,
        }
    }

    /// Get transport type as string for logging
    pub fn transport_type(&self) -> &'static str {
        match self {
            RouterForwardTargetConfig::Udp(_) => "UDP",
            RouterForwardTargetConfig::Tcp(_) => "TCP",
        }
    }

    /// Create UDP transport config
    pub fn udp(to: SocketAddr) -> Self {
        Self::Udp(UdpRouterForwardTargetConfig::new(to))
    }

    /// Create TCP transport config
    pub fn tcp(to: SocketAddr) -> Self {
        Self::Tcp(TcpRouterForwardTargetConfig::new(to))
    }
}

/// Router forward configuration
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RouterForwardConfig {
    /// The target transport to route the packet to.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub target: RouterForwardTargetConfig,

    /// Optional address rewrite for the routed packet.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub rewrite: Option<String>,
}

impl RouterForwardConfig {
    pub fn new(target: RouterForwardTargetConfig, rewrite: Option<String>) -> Self {
        Self { target, rewrite }
    }

    pub fn udp(to: SocketAddr, rewrite: Option<String>) -> Self {
        Self::new(RouterForwardTargetConfig::udp(to), rewrite)
    }

    pub fn tcp(to: SocketAddr, rewrite: Option<String>) -> Self {
        Self::new(RouterForwardTargetConfig::tcp(to), rewrite)
    }
}

/// Router route configuration
#[derive(Derivative, Debug, Clone, Getters)]
#[derivative(PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RouterRouteConfig {
    /// The OSC address pattern to match.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "serde", serde(with = "serde_regex"))]
    #[derivative(PartialEq(compare_with = "compare_regex"))]
    pub address: regex::Regex,

    /// Whether to stop propagating packets down the pipeline if matched.
    #[getset(get = "pub")]
    pub stop_propagation: bool,

    /// Forward configurations
    #[getset(get = "pub")]
    #[cfg_attr(feature = "serde", serde(default))]
    pub forward: Vec<RouterForwardConfig>,
}

fn compare_regex(a: &regex::Regex, b: &regex::Regex) -> bool {
    a.as_str() == b.as_str()
}

impl RouterRouteConfig {
    pub fn new(address: regex::Regex, stop_propagation: bool, forward: Vec<RouterForwardConfig>) -> Self {
        Self { address, stop_propagation, forward }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, Ipv4Addr};

    #[cfg(feature = "serde")]
    #[test]
    fn test_yaml_deserialization() {
        let yaml_str = r#"
            address: /avatar/parameters/(?<parameter>bHaptics_(?<position>.+)_(?<index>\d+)_bool)
            stop_propagation: false
            forward:
              - type: udp
                to: 127.0.0.1:23456
                rewrite: /$position/$index/bool

              - type: tcp
                to: 127.0.0.1:34567
        "#;

        let config: RouterRouteConfig = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(
            config,
            RouterRouteConfig {
                address: regex::Regex::new(r"/avatar/parameters/(?<parameter>bHaptics_(?<position>.+)_(?<index>\d+)_bool)").unwrap(),
                stop_propagation: false,
                forward: vec![
                    RouterForwardConfig {
                        target: RouterForwardTargetConfig::Udp(UdpRouterForwardTargetConfig {
                            to: "127.0.0.1:23456".parse().unwrap(),
                        }),
                        rewrite: Some("/$position/$index/bool".to_string()),
                    },
                    RouterForwardConfig {
                        target: RouterForwardTargetConfig::Tcp(TcpRouterForwardTargetConfig {
                            to: "127.0.0.1:34567".parse().unwrap(),
                        }),
                        rewrite: None,
                    },
                ],
            },
        )
    }

    #[test]
    fn test_transport_config_creation() {
        let udp_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000);
        
        let udp_config = RouterForwardTargetConfig::udp(udp_addr);
        assert_eq!(udp_config.remote_address(), udp_addr);
        assert_eq!(udp_config.transport_type(), "UDP");
        
        let tcp_config = RouterForwardTargetConfig::tcp(udp_addr);
        assert_eq!(tcp_config.remote_address(), udp_addr);
        assert_eq!(tcp_config.transport_type(), "TCP");
    }
}