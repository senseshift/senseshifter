use serde::{Deserialize, Serialize};
use ss_osc::server::config::{OscServerConfig, OscServerSocketConfig, RouterRouteConfig, RouterForwardConfig};
use ss_osc::server::connection_manager::ConnectionManagerConfig;
use std::net::{Ipv4Addr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OscServerModuleConfig {
    #[serde(default = "default_bool::<true>")]
    pub enabled: bool,

    pub servers: Vec<OscServerModuleInstanceConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OscServerModuleInstanceConfig {
    #[serde(default = "default_bool::<true>")]
    pub enabled: bool,

    pub server: OscServerConfig,
}

impl Default for OscServerModuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            servers: vec![
                OscServerModuleInstanceConfig {
                    enabled: true,
                    server: OscServerConfig::default()
                },
            ],
        }
    }
}

pub const fn default_bool<const V: bool>() -> bool {
    V
}
