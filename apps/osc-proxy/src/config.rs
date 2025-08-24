use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ss_osc::server::router::{OscRouterRouteRuntime, OscRouterRouteForwardRuntime};

/// Main configuration for the OSC proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Target configurations
    pub targets: HashMap<String, TargetConfig>,
    /// Routing rules
    pub routes: Vec<RouteConfig>,
    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Server listening configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// UDP addresses to bind to
    #[serde(default = "default_udp_addresses")]
    pub udp: Vec<SocketAddr>,
    /// TCP addresses to bind to (future use)
    #[serde(default)]
    pub tcp: Vec<SocketAddr>,
}

/// Target connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Transport type and address
    pub transport: TransportConfig,
    /// Reconnection interval in seconds
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval: u64,
    /// Optional description for display
    pub description: Option<String>,
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransportConfig {
    Udp {
        /// Target address
        to: SocketAddr,
    },
    Tcp {
        /// Target address
        to: SocketAddr,
    },
}

/// Routing rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// OSC address pattern (regex)
    pub pattern: String,
    /// Whether to stop processing more routes after this one matches
    #[serde(default)]
    pub stop_propagation: bool,
    /// Targets to forward to
    pub forward: Vec<ForwardConfig>,
    /// Optional description
    pub description: Option<String>,
}

/// Forward configuration for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardConfig {
    /// Target name to forward to
    pub to: String,
    /// Optional address rewriting pattern
    pub rewrite: Option<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Whether to enable file logging
    #[serde(default)]
    pub file: bool,
    /// Log file path (if file logging enabled)
    #[serde(default = "default_log_file")]
    pub file_path: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: false,
            file_path: default_log_file(),
        }
    }
}

// Default value functions
fn default_udp_addresses() -> Vec<SocketAddr> {
    vec!["127.0.0.1:8000".parse().unwrap()]
}

fn default_reconnect_interval() -> u64 {
    5
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_file() -> String {
    "osc-proxy.log".to_string()
}

impl ProxyConfig {
    /// Load configuration from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        
        let extension = path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
            
        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => {
                serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse YAML config: {:?}", path.as_ref()))
            },
            "toml" => {
                toml::from_str(&content)
                    .with_context(|| format!("Failed to parse TOML config: {:?}", path.as_ref()))
            },
            _ => {
                // Try YAML first, then TOML
                serde_yaml::from_str(&content)
                    .or_else(|_| toml::from_str(&content))
                    .with_context(|| format!("Failed to parse config as YAML or TOML: {:?}", path.as_ref()))
            }
        }
    }
    
    /// Convert to ss-osc types
    pub fn to_ss_osc_config(&self) -> Result<(Vec<ss_osc::server::connection_manager::Target>, Vec<OscRouterRouteRuntime>)> {
        // Convert targets
        let mut targets = Vec::new();
        for (name, config) in &self.targets {
            let transport = match &config.transport {
                TransportConfig::Udp { to } => {
                    ss_osc::server::connection_manager::TransportConfig::Udp(
                        ss_osc::server::connection_manager::UdpTransportConfig::new(*to)
                    )
                },
                TransportConfig::Tcp { to } => {
                    ss_osc::server::connection_manager::TransportConfig::Tcp(
                        ss_osc::server::connection_manager::TcpTransportConfig::new(*to)
                    )
                },
            };
            
            targets.push(ss_osc::server::connection_manager::Target {
                name: name.clone(),
                transport,
                reconnect_interval: Duration::from_secs(config.reconnect_interval),
            });
        }
        
        // Convert routes
        let mut routes = Vec::new();
        for route in &self.routes {
            let regex = regex::Regex::new(&route.pattern)
                .with_context(|| format!("Invalid regex pattern: {}", route.pattern))?;
                
            let forward = route.forward.iter().map(|f| OscRouterRouteForwardRuntime {
                to: f.to.clone(),
                rewrite: f.rewrite.clone(),
            }).collect();
            
            routes.push(OscRouterRouteRuntime {
                address: regex,
                stop_propagation: route.stop_propagation,
                forward,
            });
        }
        
        Ok((targets, routes))
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Check that all forward targets exist
        for route in &self.routes {
            for forward in &route.forward {
                if !self.targets.contains_key(&forward.to) {
                    return Err(anyhow::anyhow!(
                        "Route forwards to unknown target '{}'. Available targets: {:?}",
                        forward.to,
                        self.targets.keys().collect::<Vec<_>>()
                    ));
                }
            }
        }
        
        // Validate regex patterns
        for route in &self.routes {
            regex::Regex::new(&route.pattern)
                .with_context(|| format!("Invalid regex pattern in route: {}", route.pattern))?;
        }
        
        // Validate server addresses
        if self.server.udp.is_empty() && self.server.tcp.is_empty() {
            return Err(anyhow::anyhow!("At least one UDP or TCP server address must be configured"));
        }
        
        Ok(())
    }
    
    /// Create a sample configuration file
    pub fn create_sample() -> Self {
        let mut targets = HashMap::new();
        
        targets.insert("vrchat".to_string(), TargetConfig {
            transport: TransportConfig::Udp { 
                to: "127.0.0.1:9000".parse().unwrap() 
            },
            reconnect_interval: 5,
            description: Some("VRChat OSC input".to_string()),
        });
        
        targets.insert("touchdesigner".to_string(), TargetConfig {
            transport: TransportConfig::Tcp { 
                to: "127.0.0.1:9001".parse().unwrap() 
            },
            reconnect_interval: 3,
            description: Some("TouchDesigner visualization".to_string()),
        });
        
        targets.insert("ableton".to_string(), TargetConfig {
            transport: TransportConfig::Udp { 
                to: "127.0.0.1:9002".parse().unwrap() 
            },
            reconnect_interval: 10,
            description: Some("Ableton Live control".to_string()),
        });
        
        let routes = vec![
            RouteConfig {
                pattern: r"^/avatar/parameters/(?P<param>.*)$".to_string(),
                stop_propagation: false,
                forward: vec![ForwardConfig {
                    to: "vrchat".to_string(),
                    rewrite: Some("/avatar/parameters/$param".to_string()),
                }],
                description: Some("VRChat avatar parameters".to_string()),
            },
            RouteConfig {
                pattern: r"^/audio/(?P<param>.*)$".to_string(),
                stop_propagation: false,
                forward: vec![
                    ForwardConfig {
                        to: "ableton".to_string(),
                        rewrite: Some("/live/$param".to_string()),
                    },
                    ForwardConfig {
                        to: "touchdesigner".to_string(),
                        rewrite: Some("/viz/$param".to_string()),
                    },
                ],
                description: Some("Audio analysis data".to_string()),
            },
            RouteConfig {
                pattern: r"(.*)$".to_string(),
                stop_propagation: false,
                forward: vec![ForwardConfig {
                    to: "touchdesigner".to_string(),
                    rewrite: None,
                }],
                description: Some("Catchall route for monitoring".to_string()),
            },
        ];
        
        Self {
            server: ServerConfig {
                udp: vec!["127.0.0.1:8000".parse().unwrap()],
                tcp: vec![],
            },
            targets,
            routes,
            logging: LoggingConfig::default(),
        }
    }
}