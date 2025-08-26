use serde::{Deserialize, Serialize};
use ss_osc::server::config::{OscServerConfig, OscServerSocketConfig, RouterRouteConfig, RouterForwardConfig};
use ss_osc::server::connection_manager::ConnectionManagerConfig;
use std::net::{Ipv4Addr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OscServerModuleConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    pub servers: Vec<OscServerModuleInstanceConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OscServerModuleInstanceConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    pub server: OscServerConfig,
}

impl Default for OscServerModuleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            servers: vec![
                OscServerModuleInstanceConfig {
                    enabled: false,
                    server: create_default_osc_config(),
                },
                OscServerModuleInstanceConfig {
                    enabled: false, 
                    server: create_testing_osc_config(),
                }
            ],
        }
    }
}

fn default_true() -> bool {
    true
}

/// Create a default OSC server configuration
fn create_default_osc_config() -> OscServerConfig {
    OscServerConfig {
        server: OscServerSocketConfig {
            udp: vec![SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9001)],
            tcp: vec![],
        },
        routes: vec![
            RouterRouteConfig::new(
                regex::Regex::new(r"/avatar/parameters/.*").unwrap(),
                false,
                vec![
                    // UDP connection that should work
                    RouterForwardConfig::udp(
                        SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 9000),
                        None
                    ),
                ]
            )
        ],
        connection_manager: ConnectionManagerConfig::default(),
    }
}

/// Create a testing OSC server configuration with multiple connections
fn create_testing_osc_config() -> OscServerConfig {
    OscServerConfig {
        server: OscServerSocketConfig {
            udp: vec![SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9011)],
            tcp: vec![],
        },
        routes: vec![
            RouterRouteConfig::new(
                regex::Regex::new(r"/avatar/parameters/.*").unwrap(),
                false,
                vec![
                    // UDP connection that should work
                    RouterForwardConfig::udp(
                        SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 9000),
                        None
                    ),
                    // TCP connection that will likely fail/reconnect
                    RouterForwardConfig::tcp(
                        SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 9002),
                        None
                    )
                ]
            ),
            // Another route for testing multiple connections
            RouterRouteConfig::new(
                regex::Regex::new(r"/system/.*").unwrap(),
                false,
                vec![
                    // Another TCP connection to a different port
                    RouterForwardConfig::tcp(
                        SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 9003),
                        Some("/system/rewritten".to_string())
                    )
                ]
            )
        ],
        connection_manager: ConnectionManagerConfig::default(),
    }
}
