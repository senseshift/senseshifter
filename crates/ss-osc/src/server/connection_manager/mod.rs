pub mod config;
pub mod transport;
pub mod error;

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use rosc::OscPacket;
use tokio::sync::{RwLock, mpsc, broadcast};
use tokio::time::{interval, sleep, Duration};
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
    // Track reconnection attempts per target for logarithmic backoff
    reconnect_attempts: Arc<DashMap<String, u32>>,
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
            reconnect_attempts: Arc::new(DashMap::new()),
        }
    }
}

impl ConnectionManager {
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ConnectionEvent> {
        self.event_sender.subscribe()
    }

    /// Calculate reconnection delay using logarithmic backoff
    /// Starts at 0ms, then 100ms, 500ms, 1s, 2s, 5s, 10s, 20s, 30s (max)
    fn calculate_reconnect_delay(attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0); // First attempt is immediate
        }
        
        let base_delay_ms = match attempt {
            1 => 100,    // 100ms
            2 => 500,    // 500ms  
            3 => 1000,   // 1s
            4 => 2000,   // 2s
            5 => 5000,   // 5s
            6 => 10000,  // 10s
            7 => 20000,  // 20s
            _ => 30000,  // 30s max
        };
        
        Duration::from_millis(base_delay_ms)
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
        let _transport_type = target.transport.transport_type().to_string();
        
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

        // Reset reconnection attempts on successful connection start
        self.reconnect_attempts.insert(name.clone(), 0);

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
                let reconnect_attempts = self.reconnect_attempts.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    // TCP connections typically fail to localhost:9001 since nothing is listening
                    // Increment attempt counter for backoff
                    let current_attempts = reconnect_attempts.get(&target_name).map(|v| *v).unwrap_or(0);
                    reconnect_attempts.insert(target_name.clone(), current_attempts + 1);
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
        let reconnect_attempts = self.reconnect_attempts.clone();

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
                        Self::check_and_reconnect_targets(&connections, &targets, &main_cancellation_token, &event_sender, &reconnect_attempts).await;
                    }
                }
            }
        });

        *monitor_handle = Some(handle);
        Ok(())
    }

    async fn check_and_reconnect_targets(
        connections: &Arc<DashMap<String, ConnectionState>>,
        targets: &Arc<DashMap<String, Target>>,
        main_cancellation_token: &CancellationToken,
        event_sender: &broadcast::Sender<ConnectionEvent>,
        reconnect_attempts: &Arc<DashMap<String, u32>>,
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

        // Mark failed connections and schedule reconnection with backoff
        for name in failed_connections {
            let current_attempts = reconnect_attempts.get(&name).map(|v| *v).unwrap_or(0);
            let next_attempt = current_attempts + 1;
            let delay = Self::calculate_reconnect_delay(next_attempt);
            
            warn!("Connection to {} failed, scheduling reconnection attempt {} in {:?}", 
                  name, next_attempt, delay);
            
            connections.insert(name.clone(), ConnectionState::Failed);
            let _ = event_sender.send(ConnectionEvent::Failed { target_name: name.clone() });
            
            // Schedule reconnection with logarithmic backoff
            if let Some(target_ref) = targets.get(&name) {
                let target = target_ref.value().clone();
                let connections_clone = Arc::clone(connections);
                let event_sender_clone = event_sender.clone();
                let reconnect_attempts_clone = Arc::clone(reconnect_attempts);
                let cancellation_token = main_cancellation_token.child_token();
                
                tokio::spawn(async move {
                    // Wait for backoff delay
                    if delay > Duration::from_millis(0) {
                        tokio::select! {
                            _ = sleep(delay) => {},
                            _ = cancellation_token.cancelled() => return,
                        }
                    }
                    
                    // Update attempt counter
                    reconnect_attempts_clone.insert(name.clone(), next_attempt);
                    
                    // Emit reconnecting event
                    let _ = event_sender_clone.send(ConnectionEvent::Reconnecting { target_name: name.clone() });
                    
                    // Attempt reconnection
                    Self::attempt_single_reconnection(
                        &name,
                        &target,
                        &connections_clone,
                        &event_sender_clone,
                        &reconnect_attempts_clone,
                        cancellation_token,
                    ).await;
                });
            }
        }
    }

    async fn attempt_single_reconnection(
        target_name: &str,
        target: &Target,
        connections: &Arc<DashMap<String, ConnectionState>>,
        event_sender: &broadcast::Sender<ConnectionEvent>,
        reconnect_attempts: &Arc<DashMap<String, u32>>,
        cancellation_token: CancellationToken,
    ) {
        info!("Attempting to reconnect to target: {}", target_name);
        
        let (packet_tx, packet_rx) = mpsc::channel(1000);
        let connection_token = cancellation_token.child_token();

        let transport_handler = create_transport_handler(&target.transport);
        let handler_clone = transport_handler.clone();
        let connection_token_clone = connection_token.clone();
        let event_sender_clone = event_sender.clone();
        let target_name_clone = target_name.to_string();
        let reconnect_attempts_clone = reconnect_attempts.clone();
        
        let handle = tokio::spawn(async move {
            match handler_clone.start(packet_rx, connection_token_clone).await {
                Ok(_) => {
                    info!("Transport handler for {} completed gracefully", target_name_clone);
                    let _ = event_sender_clone.send(ConnectionEvent::Disconnected { target_name: target_name_clone.clone() });
                },
                Err(e) => {
                    error!("Transport handler for {} failed: {}", target_name_clone, e);
                    // Increment attempts and schedule next reconnection
                    let current_attempts = reconnect_attempts_clone.get(&target_name_clone).map(|v| *v).unwrap_or(0);
                    reconnect_attempts_clone.insert(target_name_clone.clone(), current_attempts + 1);
                    let _ = event_sender_clone.send(ConnectionEvent::Failed { target_name: target_name_clone });
                }
            }
        });

        // Store the new connection
        connections.insert(
            target_name.to_string(),
            ConnectionState::Connected {
                sender: packet_tx,
                handle,
                cancellation_token: connection_token,
            },
        );

        // Check if connection succeeds based on transport type
        match &target.transport {
            TransportConfig::Udp(_) => {
                // UDP is connectionless, consider it successful
                info!("UDP target {} reconnected successfully", target_name);
                reconnect_attempts.insert(target_name.to_string(), 0); // Reset attempts
                let _ = event_sender.send(ConnectionEvent::Connected { target_name: target_name.to_string() });
            },
            TransportConfig::Tcp(_) => {
                // TCP requires actual connection - wait and see if it fails
                let event_sender_clone = event_sender.clone();
                let target_name_clone = target_name.to_string();
                let reconnect_attempts_clone = reconnect_attempts.clone();
                
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    // Check if connection is still alive (simplified check)
                    // In reality, we'd need proper connection health checking
                    // For now, assume TCP to localhost:9001 fails
                    let current_attempts = reconnect_attempts_clone.get(&target_name_clone).map(|v| *v).unwrap_or(0);
                    reconnect_attempts_clone.insert(target_name_clone.clone(), current_attempts + 1);
                    let _ = event_sender_clone.send(ConnectionEvent::Failed { target_name: target_name_clone });
                });
            }
        }
    }

    // Old reconnection method - removed in favor of logarithmic backoff approach
    // Reconnection is now handled automatically by check_and_reconnect_targets with backoff
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