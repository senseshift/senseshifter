pub mod config;
pub mod transport;
pub mod error;

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use rosc::OscPacket;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, sleep};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

pub use config::{
    ConnectionManagerConfig, Target, TransportConfig,
    UdpTransportConfig, TcpTransportConfig,
    // Legacy re-exports for backward compatibility
    Target as OscTarget,
    TransportConfig as OscTransportConfig,
    UdpTransportConfig as OscTransportUdpConfig,
    TcpTransportConfig as OscTransportTcpConfig,
};
pub use error::{ConnectionError, ConnectionResult};
use transport::create_transport_handler;


#[derive(Debug)]
enum ConnectionState {
    Connected {
        sender: mpsc::Sender<OscPacket>,
        handle: tokio::task::JoinHandle<()>,
        cancellation_token: CancellationToken,
    },
    Connecting,
    Disconnected,
    Failed,
}

pub struct ConnectionManager {
    config: ConnectionManagerConfig,
    targets: Arc<DashMap<String, Target>>,
    connections: Arc<DashMap<String, ConnectionState>>,
    // Persistent proxy senders that forward to current active connections
    proxy_senders: Arc<DashMap<String, mpsc::Sender<OscPacket>>>,
    monitor_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    cancellation_token: CancellationToken,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::with_config(ConnectionManagerConfig::default())
    }

    pub fn with_config(config: ConnectionManagerConfig) -> Self {
        Self {
            config,
            targets: Arc::new(DashMap::new()),
            connections: Arc::new(DashMap::new()),
            proxy_senders: Arc::new(DashMap::new()),
            monitor_handle: Arc::new(RwLock::new(None)),
            cancellation_token: CancellationToken::new(),
        }
    }
}

impl ConnectionManager {
    pub fn add_target(&self, target: Target) -> ConnectionResult<mpsc::Sender<OscPacket>> {
        let name = target.name.clone();
        
        if self.targets.contains_key(&name) {
            return Err(ConnectionError::TargetExists(name));
        }
        
        self.targets.insert(name.clone(), target);
        self.connections.insert(name.clone(), ConnectionState::Disconnected);

        // Create a persistent proxy sender for this target
        let (proxy_tx, mut proxy_rx) = mpsc::channel(self.config.packet_buffer_size);
        self.proxy_senders.insert(name.clone(), proxy_tx.clone());

        // Start a forwarding task that routes packets to the current active connection
        let connections = Arc::clone(&self.connections);
        let target_name = name.clone();
        let cancellation_token = self.cancellation_token.child_token();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        debug!("Proxy sender task cancelled for {}", target_name);
                        break;
                    }
                    Some(packet) = proxy_rx.recv() => {
                        // Forward to current active connection
                        if let Some(connection_ref) = connections.get(&target_name) {
                            if let ConnectionState::Connected { sender, .. } = connection_ref.value() {
                                if let Err(e) = sender.send(packet).await {
                                    debug!("Failed to forward packet to {} (connection may be down): {}", target_name, e);
                                }
                            } else {
                                debug!("No active connection for {}, dropping packet", target_name);
                            }
                        }
                    }
                    else => break,
                }
            }
            debug!("Proxy sender task ended for {}", target_name);
        });

        Ok(proxy_tx)
    }

    pub fn remove_target(&self, name: &str) -> Option<Target> {
        // Cancel the connection if it exists
        if let Some((_, connection)) = self.connections.remove(name) {
            if let ConnectionState::Connected { cancellation_token, .. } = connection {
                cancellation_token.cancel();
            }
        }

        // Remove proxy sender (this will cause the forwarding task to end)
        self.proxy_senders.remove(name);

        // Remove from targets
        self.targets.remove(name).map(|(_, target)| target)
    }

    pub async fn connect_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for target_ref in self.targets.iter() {
            let target = target_ref.value().clone();
            self.connect_target(target).await?;
        }
        Ok(())
    }

    async fn connect_target(&self, target: Target) -> Result<(), Box<dyn std::error::Error>> {
        let name = target.name.clone();

        // Check if already connected/connecting
        if let Some(state) = self.connections.get(&name) {
            match state.value() {
                ConnectionState::Connected { .. } | ConnectionState::Connecting => {
                    return Ok(());
                }
                _ => {}
            }
        }

        // Mark as connecting
        self.connections.insert(name.clone(), ConnectionState::Connecting);

        let remote_addr = target.transport.remote_address();
        debug!("Connecting to OSC target: {} ({} -> {})",
               name, target.transport.transport_type(), remote_addr);

        let (packet_tx, packet_rx) = mpsc::channel(self.config.packet_buffer_size);
        let connection_token = self.cancellation_token.child_token();

        let transport_handler = create_transport_handler(&target.transport);
        let handler_clone = transport_handler.clone();
        let connection_token_clone = connection_token.clone();
        
        let handle = tokio::spawn(async move {
            let _ = handler_clone.start(packet_rx, connection_token_clone).await;
        });

        // Store the connection
        self.connections.insert(
            name.clone(),
            ConnectionState::Connected {
                sender: packet_tx,
                handle,
                cancellation_token: connection_token,
            },
        );

        info!("Connected to OSC target: {} ({} -> {})",
              name, target.transport.transport_type(), remote_addr);
        Ok(())
    }


    pub fn get_forward_targets(&self) -> HashMap<String, mpsc::Sender<OscPacket>> {
        // Return proxy senders that persist across reconnections
        let mut targets = HashMap::new();

        for proxy_ref in self.proxy_senders.iter() {
            let name = proxy_ref.key().clone();
            let sender = proxy_ref.value().clone();
            targets.insert(name, sender);
        }

        targets
    }

    pub async fn shutdown(&self) {
        self.cancellation_token.cancel();

        // Wait for monitor to finish if it's running
        let monitor_handle_opt = {
            let mut monitor_handle = self.monitor_handle.write().await;
            monitor_handle.take()
        };

        if let Some(handle) = monitor_handle_opt {
            let _ = handle.await;
        }
    }

    pub async fn start_connection_monitor(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut monitor_handle = self.monitor_handle.write().await;
        if monitor_handle.is_some() {
            return Err("Connection monitor already started".into());
        }

        let connections = Arc::clone(&self.connections);
        let targets = Arc::clone(&self.targets);
        let main_cancellation_token = self.cancellation_token.clone();

        let health_check_duration = self.config.health_check_interval;
        let handle = tokio::spawn(async move {
            let mut health_check_interval = interval(health_check_duration);

            loop {
                tokio::select! {
                    // Handle cancellation
                    _ = main_cancellation_token.cancelled() => {
                        info!("Connection manager shutting down");
                        break;
                    }

                    // Periodic health checks and reconnection attempts
                    _ = health_check_interval.tick() => {
                        Self::check_and_reconnect_targets(&connections, &targets, &main_cancellation_token).await;
                    }
                }
            }
        });

        *monitor_handle = Some(handle);
        Ok(())
    }

    async fn check_and_reconnect_targets(
        connections: &DashMap<String, ConnectionState>,
        _targets: &DashMap<String, Target>,
        _main_cancellation_token: &CancellationToken,
    ) {
        // Check for finished/failed connections
        let mut failed_connections = Vec::new();

        for connection_ref in connections.iter() {
            let name = connection_ref.key();
            if let ConnectionState::Connected { handle, .. } = connection_ref.value() {
                if handle.is_finished() {
                    failed_connections.push(name.clone());
                }
            }
        }

        // Mark failed connections and attempt reconnection
        for name in failed_connections {
            warn!("Connection to {} failed, attempting reconnection", name);
            connections.insert(name, ConnectionState::Failed);
        }

        // Self::reconnect_failed_targets(connections, targets, main_cancellation_token).await;
    }

    async fn reconnect_failed_targets(
        connections: &DashMap<String, ConnectionState>,
        targets: &DashMap<String, Target>,
        main_cancellation_token: &CancellationToken,
    ) {
        let failed_connections: Vec<String> = connections
            .iter()
            .filter_map(|entry| {
                match entry.value() {
                    ConnectionState::Failed | ConnectionState::Disconnected => {
                        Some(entry.key().clone())
                    }
                    _ => None,
                }
            })
            .collect();

        for name in failed_connections {
            if let Some(target_ref) = targets.get(&name) {
                let target = target_ref.value().clone();
                info!("Attempting to reconnect to {}", name);

                // Wait before reconnecting
                sleep(target.reconnect_interval).await;

                // Attempt reconnection
                let (packet_tx, packet_rx) = tokio::sync::mpsc::channel(1000);
                let connection_token = main_cancellation_token.child_token();

                let transport_handler = create_transport_handler(&target.transport);
                let handler_clone = transport_handler.clone();
                let connection_token_clone = connection_token.clone();
                
                let handle = tokio::spawn(async move {
                    let _ = handler_clone.start(packet_rx, connection_token_clone).await;
                });

                connections.insert(
                    name.clone(),
                    ConnectionState::Connected {
                        sender: packet_tx,
                        handle,
                        cancellation_token: connection_token,
                    },
                );

                info!("Reconnected to OSC target: {}", name);
            }
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_connection_manager_creation() {
        let manager = ConnectionManager::new();
        assert_eq!(manager.targets.len(), 0);
        assert_eq!(manager.connections.len(), 0);
    }

    #[tokio::test]
    async fn test_add_remove_target() {
        let manager = ConnectionManager::new();

        let result = manager.add_target(Target {
            name: "test".to_string(),
            transport: TransportConfig::Udp(UdpTransportConfig {
                to: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000),
            }),
            reconnect_interval: Duration::from_secs(5),
        });
        
        assert!(result.is_ok());
        
        assert_eq!(manager.targets.len(), 1);
        assert_eq!(manager.connections.len(), 1);

        let removed = manager.remove_target("test");

        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "test");
        
        assert_eq!(manager.targets.len(), 0);
        assert_eq!(manager.connections.len(), 0);
    }
}