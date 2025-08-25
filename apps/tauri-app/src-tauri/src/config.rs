use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use ss_osc::server::config::OscServerConfig;
use uuid::Uuid;

/// Configuration for a single OSC server instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OscServerInstanceConfig {
    /// Unique identifier for this server instance
    pub id: Uuid,
    /// Display name for this server instance
    pub name: String,
    /// Whether this server instance is enabled
    pub enabled: bool,
    /// The OSC server configuration
    pub config: OscServerConfig,
}

impl OscServerInstanceConfig {
    pub fn new(name: String, config: OscServerConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            enabled: false,
            config,
        }
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// Global enable/disable switch for all OSC functionality
    pub osc_enabled: bool,
    /// Map of OSC server instances by their UUID
    pub osc_servers: HashMap<Uuid, OscServerInstanceConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut osc_servers = HashMap::new();
        
        // Create a default server instance with hardcoded configuration
        let default_config = create_default_osc_config();
        let instance = OscServerInstanceConfig::new("Default Server".to_string(), default_config);
        osc_servers.insert(instance.id, instance);
        
        Self {
            osc_enabled: false,
            osc_servers,
        }
    }
}

/// Create a default OSC server configuration
fn create_default_osc_config() -> OscServerConfig {
    use ss_osc::server::config::{OscServerSocketConfig, RouterRouteConfig, RouterForwardConfig};
    use ss_osc::server::connection_manager::ConnectionManagerConfig;
    use std::net::{Ipv4Addr, SocketAddr};
    
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

impl AppConfig {
    /// Add a new OSC server instance
    pub fn add_osc_server(&mut self, name: String, config: OscServerConfig) -> Uuid {
        let instance = OscServerInstanceConfig::new(name, config);
        let id = instance.id;
        self.osc_servers.insert(id, instance);
        id
    }
    
    /// Remove an OSC server instance
    pub fn remove_osc_server(&mut self, id: &Uuid) -> Option<OscServerInstanceConfig> {
        self.osc_servers.remove(id)
    }
    
    /// Get a specific OSC server instance
    pub fn get_osc_server(&self, id: &Uuid) -> Option<&OscServerInstanceConfig> {
        self.osc_servers.get(id)
    }
    
    /// Get a mutable reference to a specific OSC server instance
    pub fn get_osc_server_mut(&mut self, id: &Uuid) -> Option<&mut OscServerInstanceConfig> {
        self.osc_servers.get_mut(id)
    }
    
    /// Toggle the enabled state of a specific server instance
    pub fn toggle_osc_server(&mut self, id: &Uuid) -> Option<bool> {
        if let Some(server) = self.osc_servers.get_mut(id) {
            server.enabled = !server.enabled;
            Some(server.enabled)
        } else {
            None
        }
    }
}