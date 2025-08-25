use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Context;
use derivative::Derivative;
use getset::Getters;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use crate::server::connection_manager::ConnectionManagerConfig;
pub use crate::server::router::config::{
    RouterRouteConfig,
    RouterForwardConfig,
    RouterForwardTargetConfig,
};

/// Main OSC server configuration with integrated routing
#[derive(Derivative, Getters)]
#[derivative(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OscServerConfig {
    /// Server UDP address to bind to
    #[getset(get = "pub")]
    #[cfg_attr(feature = "serde", serde(default))]
    pub server: OscServerSocketConfig,

    /// Routing rules with direct address forwarding
    #[getset(get = "pub")]
    #[cfg_attr(feature = "serde", serde(default))]
    pub routes: Vec<RouterRouteConfig>,

    #[getset(get = "pub")]
    #[cfg_attr(feature = "serde", serde(default))]
    pub connection_manager: ConnectionManagerConfig,
}

#[derive(Derivative, Clone, PartialEq, Eq)]
#[derivative(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OscServerSocketConfig {
    #[cfg_attr(feature = "serde", serde(default))]
    pub udp: Vec<SocketAddr>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub tcp: Vec<SocketAddr>,
}

impl Default for OscServerSocketConfig {
    fn default() -> Self {
        Self {
            udp: vec![
                SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 6001),
            ],
            tcp: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_yaml() {
        let yaml_str = r#"
            server:
              udp:
                - 127.0.0.1:9002
              tcp:
                - 127.0.0.1:9003

            routes:
              - address: /test/address
                stop_propagation: true
                forward:
                  - type: udp
                    to: 10.10.10.1:9003
        "#;

        let config: OscServerConfig = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(config, OscServerConfig {
            server: OscServerSocketConfig {
                udp: vec![
                    "127.0.0.1:9002".parse().unwrap(),
                ],
                tcp: vec![
                    "127.0.0.1:9003".parse().unwrap(),
                ],
            },
            routes: vec![
                RouterRouteConfig {
                    address: regex::Regex::new(r"/test/address").unwrap(),
                    stop_propagation: true,
                    forward: vec![
                        RouterForwardConfig {
                            target: RouterForwardTargetConfig::udp("10.10.10.1:9003".parse().unwrap()),
                            rewrite: None,
                        },
                    ],
                },
            ],
            connection_manager: ConnectionManagerConfig::default(),
        });
    }
}