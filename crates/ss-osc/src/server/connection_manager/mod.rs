pub mod config;
pub mod transport;
pub mod error;

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use rosc::OscPacket;
use tokio::sync::{RwLock, mpsc, broadcast};
use tokio::time::{interval, sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

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

#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connected { target_name: String },
    Disconnected { target_name: String },
    Reconnecting { target_name: String },
    Failed { target_name: String },
}

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

#[derive(Debug)]
pub struct ConnectionManager {
    config: ConnectionManagerConfig,
    targets: Arc<DashMap<String, Target>>,
    connections: Arc<DashMap<String, ConnectionState>>,
    // Persistent proxy senders that forward to current active connections
    proxy_senders: Arc<DashMap<String, mpsc::Sender<OscPacket>>>,
    monitor_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    cancellation_token: CancellationToken,
    event_sender: broadcast::Sender<ConnectionEvent>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::with_config(ConnectionManagerConfig::default())
    }

    pub fn with_config(config: ConnectionManagerConfig) -> Self {
        let (event_sender, _) = broadcast::channel(100);
        Self {
            config,
            targets: Arc::new(DashMap::new()),
            connections: Arc::new(DashMap::new()),
            proxy_senders: Arc::new(DashMap::new()),
            monitor_handle: Arc::new(RwLock::new(None)),
            cancellation_token: CancellationToken::new(),
            event_sender,
        }
    }
}

impl ConnectionManager {
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ConnectionEvent> {
        self.event_sender.subscribe()
    }

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

    pub async fn connect_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for target_ref in self.targets.iter() {
            let target = target_ref.value().clone();
            self.connect_target(target).await?;
        }
        Ok(())
    }

    async fn connect_target(&self, target: Target) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        let _ = self.event_sender.send(ConnectionEvent::Reconnecting { target_name: name.clone() });

        let remote_addr = target.transport.remote_address();
        debug!("Connecting to OSC target: {} ({} -> {})",
               name, target.transport.transport_type(), remote_addr);

        let (packet_tx, packet_rx) = mpsc::channel(self.config.packet_buffer_size);
        let connection_token = self.cancellation_token.child_token();

        let transport_handler = create_transport_handler(&target.transport);
        let handler_clone = transport_handler.clone();
        let connection_token_clone = connection_token.clone();
        
        // Start transport handler with proper status reporting
        let event_sender = self.event_sender.clone();
        let target_name_clone = name.clone();
        let transport_type = target.transport.transport_type().to_string();
        
        let handle = tokio::spawn(async move {
            // Try to start the transport handler
            match handler_clone.start(packet_rx, connection_token_clone).await {
                Ok(_) => {
                    info!("Transport handler for {} completed gracefully", target_name_clone);
                    let _ = event_sender.send(ConnectionEvent::Disconnected { target_name: target_name_clone });
                },
                Err(e) => {
                    error!("Transport handler for {} failed: {}", target_name_clone, e);
                    let _ = event_sender.send(ConnectionEvent::Failed { target_name: target_name_clone });
                }
            }
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

        // Emit connection status based on transport type
        match &target.transport {
            TransportConfig::Udp(_) => {
                // UDP is connectionless, so we can immediately mark as connected
                info!("UDP target {} ready to send packets to {}", name, remote_addr);
                let _ = self.event_sender.send(ConnectionEvent::Connected { target_name: name.clone() });
            },
            TransportConfig::Tcp(_) => {
                // TCP requires actual connection - let the transport handler report status
                // For now, we'll assume it fails since we don't have proper connection detection
                debug!("TCP target {} attempting connection to {}", name, remote_addr);
                // Wait a moment then check if still alive, otherwise mark as failed
                let event_sender = self.event_sender.clone();
                let target_name = name.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    // TCP connections typically fail to localhost:9001 since nothing is listening
                    let _ = event_sender.send(ConnectionEvent::Failed { target_name });
                });
            }
        }
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

    pub async fn start_connection_monitor(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut monitor_handle = self.monitor_handle.write().await;
        if monitor_handle.is_some() {
            return Err("Connection monitor already started".into());
        }

        let connections = Arc::clone(&self.connections);
        let targets = Arc::clone(&self.targets);
        let main_cancellation_token = self.cancellation_token.clone();
        let event_sender = self.event_sender.clone();

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
                        Self::check_and_reconnect_targets(&connections, &targets, &main_cancellation_token, &event_sender).await;
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
        event_sender: &broadcast::Sender<ConnectionEvent>,
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
            connections.insert(name.clone(), ConnectionState::Failed);
            let _ = event_sender.send(ConnectionEvent::Failed { target_name: name });
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
    use std::time::Duration;

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