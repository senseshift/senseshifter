use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock, oneshot};
use tracing::{info, error, warn};
use uuid::Uuid;

use ss_osc::server::{OscServerBuilder, OscServer};
use ss_osc::server::connection_manager::ConnectionEvent;

use super::config::{OscServerModuleConfig, OscServerModuleInstanceConfig};

/// Runtime configuration for a single OSC server instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OscServerRuntimeConfig {
    /// Unique identifier for this server instance
    pub id: Uuid,
    /// Display name for this server instance
    pub name: String,
    /// Whether this server instance is enabled
    pub enabled: bool,
    /// The OSC server configuration
    pub config: ss_osc::server::config::OscServerConfig,
}

/// Connection status for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionStatus {
    Online,
    Offline,
    Reconnecting,
    Failed,
}

/// Event sent to UI for server status updates
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatusEvent {
    pub server_id: Uuid,
    pub server_name: String,
    pub enabled: bool,
    pub running: bool,
}

/// Event sent to UI for connection status updates
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatusEvent {
    pub server_id: Uuid,
    pub target_name: String,
    pub address: String,
    pub transport: String,
    pub status: ConnectionStatus,
    pub next_attempt_at: Option<i64>,
}

/// Runtime configuration combining all server instances
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OscRuntimeConfig {
    /// Global enable/disable switch for all OSC functionality
    pub osc_enabled: bool,
    /// Map of OSC server instances by their UUID
    pub osc_servers: HashMap<Uuid, OscServerRuntimeConfig>,
}

impl Default for OscRuntimeConfig {
    fn default() -> Self {
        Self {
            osc_enabled: false,
            osc_servers: HashMap::new(),
        }
    }
}

/// Internal server instance state
struct ServerInstance {
    config: OscServerRuntimeConfig,
    server: Option<OscServer>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

/// OSC Server Manager that handles multiple server instances
pub struct OscServerManager {
    servers: Arc<RwLock<HashMap<Uuid, ServerInstance>>>,
    event_tx: mpsc::UnboundedSender<serde_json::Value>,
    config: Arc<RwLock<OscRuntimeConfig>>,
}

impl OscServerManager {
    pub fn new(event_tx: mpsc::UnboundedSender<serde_json::Value>) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            config: Arc::new(RwLock::new(OscRuntimeConfig::default())),
        }
    }

    /// Generate runtime configuration from module configuration
    pub fn generate_runtime_config(module_config: OscServerModuleConfig) -> OscRuntimeConfig {
        let mut osc_servers = HashMap::new();
        
        for (index, instance_config) in module_config.servers.into_iter().enumerate() {
            let server_id = Uuid::new_v4();
            let server_name = format!("OSC Server {}", index + 1);
            
            let runtime_config = OscServerRuntimeConfig {
                id: server_id,
                name: server_name,
                enabled: instance_config.enabled,
                config: instance_config.server,
            };
            
            osc_servers.insert(server_id, runtime_config);
        }
        
        OscRuntimeConfig {
            osc_enabled: module_config.enabled,
            osc_servers,
        }
    }

    /// Initialize the manager with runtime config
    pub async fn initialize(&self, config: OscRuntimeConfig) -> Result<()> {
        info!("Initializing OSC server manager with config: {:?}", config);
        
        let mut config_guard = self.config.write().await;
        *config_guard = config.clone();
        drop(config_guard);

        // Initialize all server instances from config
        let mut servers = self.servers.write().await;
        for (id, server_config) in config.osc_servers {
            info!("Adding server instance: {} ({})", server_config.name, id);
            servers.insert(id, ServerInstance {
                config: server_config,
                server: None,
                shutdown_tx: None,
            });
        }

        info!("OSC server manager initialization completed");
        Ok(())
    }

    /// Get current runtime config
    pub async fn get_config(&self) -> OscRuntimeConfig {
        let config = self.config.read().await.clone();
        info!("get_config returning: {:?}", config);
        config
    }

    /// Toggle global OSC functionality
    pub async fn toggle_global_osc(&self, enabled: bool) -> Result<()> {
        let mut config = self.config.write().await;
        config.osc_enabled = enabled;
        
        if enabled {
            // Start all enabled server instances
            let enabled_servers: Vec<_> = config.osc_servers
                .iter()
                .filter(|(_, server)| server.enabled)
                .map(|(id, _)| *id)
                .collect();
            
            drop(config);
            
            for server_id in enabled_servers {
                if let Err(e) = self.start_server(server_id).await {
                    error!("Failed to start server {}: {}", server_id, e);
                }
            }
        } else {
            drop(config);
            // Stop all running servers
            self.stop_all_servers().await?;
        }

        Ok(())
    }

    /// Toggle a specific server instance
    pub async fn toggle_server(&self, server_id: Uuid, enabled: bool) -> Result<()> {
        let config = self.config.read().await;
        let global_enabled = config.osc_enabled;
        drop(config);

        // Update server config
        {
            let mut config = self.config.write().await;
            if let Some(server_config) = config.osc_servers.get_mut(&server_id) {
                server_config.enabled = enabled;
            } else {
                return Err(anyhow::anyhow!("Server {} not found", server_id));
            }
        }

        if global_enabled && enabled {
            self.start_server(server_id).await?;
        } else {
            self.stop_server(server_id).await?;
        }

        // Send server status event immediately
        self.send_server_status_event(server_id).await;
        Ok(())
    }

    /// Start a specific server instance
    async fn start_server(&self, server_id: Uuid) -> Result<()> {
        let mut servers = self.servers.write().await;
        let server_instance = servers.get_mut(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?;

        // Don't start if already running
        if server_instance.server.is_some() {
            return Ok(());
        }

        info!("Starting OSC server instance: {}", server_instance.config.name);

        // Build and start the server
        let builder = OscServerBuilder::new(server_instance.config.config.clone());
        let server = builder.build()?;

        // Setup event handling
        let (_server_events_rx, mut conn_events_rx) = server.subscribe_to_events();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

        let event_tx = self.event_tx.clone();
        let server_name = server_instance.config.name.clone();

        // Spawn event handler task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = conn_events_rx.recv() => {
                        match event {
                            Ok(conn_event) => {
                                let (status, next_attempt_at) = match &conn_event {
                                    ConnectionEvent::Connected { .. } => {
                                        (ConnectionStatus::Online, None)
                                    }
                                    ConnectionEvent::Disconnected { .. } => {
                                        (ConnectionStatus::Offline, None)
                                    }
                                    ConnectionEvent::Reconnecting { .. } => {
                                        (ConnectionStatus::Reconnecting, None)
                                    }
                                    ConnectionEvent::Failed { next_attempt_at, .. } => {
                                        let duration_from_now = next_attempt_at.saturating_duration_since(std::time::Instant::now());
                                        let seconds_from_now = duration_from_now.as_secs() as i64;
                                        (ConnectionStatus::Failed, Some(seconds_from_now))
                                    }
                                };

                                let target = match &conn_event {
                                    ConnectionEvent::Connected { target } |
                                    ConnectionEvent::Disconnected { target } |
                                    ConnectionEvent::Reconnecting { target } |
                                    ConnectionEvent::Failed { target, .. } => target,
                                };

                                let event = ConnectionStatusEvent {
                                    server_id,
                                    target_name: target.name.clone(),
                                    address: target.transport.remote_address(),
                                    transport: target.transport.transport_type().to_string(),
                                    status,
                                    next_attempt_at,
                                };

                                if let Ok(json_event) = serde_json::to_value(&event) {
                                    if let Err(e) = event_tx.send(json_event) {
                                        error!("Failed to send connection status event: {}", e);
                                    }
                                }
                            }
                            Err(_) => {
                                warn!("Connection event channel closed for server {}", server_name);
                                break;
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        info!("Event handler shutdown for server {}", server_name);
                        break;
                    }
                }
            }
        });

        // Store server and shutdown channel
        server_instance.server = Some(server);
        server_instance.shutdown_tx = Some(shutdown_tx);

        // Release the lock before sending event to avoid potential deadlock
        drop(servers);
        self.send_server_status_event(server_id).await;
        Ok(())
    }

    /// Stop a specific server instance
    async fn stop_server(&self, server_id: Uuid) -> Result<()> {
        let mut servers = self.servers.write().await;
        let server_instance = servers.get_mut(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?;

        if let Some(server) = server_instance.server.take() {
            info!("Stopping OSC server instance: {}", server_instance.config.name);
            
            // Send shutdown signal to event handler
            if let Some(shutdown_tx) = server_instance.shutdown_tx.take() {
                let _ = shutdown_tx.send(());
            }
            
            // Shutdown the server
            server.shutdown().await;
        }

        // Release the lock before sending event to avoid potential deadlock
        drop(servers);
        self.send_server_status_event(server_id).await;
        Ok(())
    }

    /// Stop all running servers
    async fn stop_all_servers(&self) -> Result<()> {
        let servers = self.servers.read().await;
        let server_ids: Vec<_> = servers.keys().cloned().collect();
        drop(servers);

        for server_id in server_ids {
            if let Err(e) = self.stop_server(server_id).await {
                error!("Failed to stop server {}: {}", server_id, e);
            }
        }

        Ok(())
    }

    /// Send server status event to UI
    async fn send_server_status_event(&self, server_id: Uuid) {
        info!("Sending server status event for server {}", server_id);
        let servers = self.servers.read().await;
        
        if let Some(server_instance) = servers.get(&server_id) {
            let event = ServerStatusEvent {
                server_id,
                server_name: server_instance.config.name.clone(),
                enabled: server_instance.config.enabled,
                running: server_instance.server.is_some(),
            };

            info!("Server status event: {:?}", event);

            if let Ok(json_event) = serde_json::to_value(&event) {
                info!("Serialized server status event: {:?}", json_event);
                if let Err(e) = self.event_tx.send(json_event) {
                    error!("Failed to send server status event: {}", e);
                } else {
                    info!("Server status event sent successfully to channel");
                }
            } else {
                error!("Failed to serialize server status event");
            }
        } else {
            warn!("Server {} not found when trying to send status event", server_id);
        }
    }

    /// Get all server statuses
    pub async fn get_all_server_statuses(&self) -> Vec<ServerStatusEvent> {
        let servers = self.servers.read().await;
        let mut statuses = Vec::new();

        for (server_id, server_instance) in servers.iter() {
            statuses.push(ServerStatusEvent {
                server_id: *server_id,
                server_name: server_instance.config.name.clone(),
                enabled: server_instance.config.enabled,
                running: server_instance.server.is_some(),
            });
        }

        statuses
    }
}