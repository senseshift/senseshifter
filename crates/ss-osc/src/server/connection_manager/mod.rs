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
    Connected { target: Target },
    Disconnected { target: Target },
    Reconnecting { target: Target },
    Failed { target: Target, next_attempt_at: std::time::SystemTime },
}

#[derive(Debug, Clone)]
pub enum ConnectionCommand {
    ManualReconnect { target_name: String },
    Disconnect { target_name: String },
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
    command_sender: mpsc::Sender<ConnectionCommand>,
    command_receiver: Arc<RwLock<Option<mpsc::Receiver<ConnectionCommand>>>>,
    // Track reconnection attempts per target for logarithmic backoff
    reconnect_attempts: Arc<DashMap<String, u32>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::with_config(ConnectionManagerConfig::default())
    }

    pub fn with_config(config: ConnectionManagerConfig) -> Self {
        let (event_sender, _) = broadcast::channel(100);
        let (command_sender, command_receiver) = mpsc::channel(100);
        Self {
            config,
            targets: Arc::new(DashMap::new()),
            connections: Arc::new(DashMap::new()),
            proxy_senders: Arc::new(DashMap::new()),
            monitor_handle: Arc::new(RwLock::new(None)),
            cancellation_token: CancellationToken::new(),
            event_sender,
            command_sender,
            command_receiver: Arc::new(RwLock::new(Some(command_receiver))),
            reconnect_attempts: Arc::new(DashMap::new()),
        }
    }
}

impl ConnectionManager {
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ConnectionEvent> {
        self.event_sender.subscribe()
    }

    pub fn get_command_sender(&self) -> mpsc::Sender<ConnectionCommand> {
        self.command_sender.clone()
    }

    pub async fn send_command(&self, command: ConnectionCommand) -> Result<(), mpsc::error::SendError<ConnectionCommand>> {
        self.command_sender.send(command).await
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

    // Removed handle_transport_failure - now handled by self-reconnecting transport tasks

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

    pub fn add_targets(&self, targets: Vec<Target>) -> ConnectionResult<HashMap<String, mpsc::Sender<OscPacket>>> {
        let mut senders = HashMap::new();

        for target in targets {
            let name = target.name.clone();
            if self.targets.contains_key(&name) {
                return Err(ConnectionError::TargetExists(name));
            }
            let sender = self.add_target(target)?;
            senders.insert(name, sender);
        }

        Ok(senders)
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
        let _ = self.event_sender.send(ConnectionEvent::Reconnecting { target: target.clone() });

        let remote_addr = target.transport.remote_address();
        debug!("Connecting to OSC target: {} ({} -> {})",
               name, target.transport.transport_type(), remote_addr);

        let (packet_tx, packet_rx) = mpsc::channel(self.config.packet_buffer_size);
        let connection_token = self.cancellation_token.child_token();

        let transport_handler = create_transport_handler(&target.transport);
        let mut handler_clone = transport_handler.clone();
        let connection_token_clone = connection_token.clone();
        
        // Start transport handler with proper status reporting
        let event_sender = self.event_sender.clone();
        let target_name_clone = name.clone();
        let _transport_type = target.transport.transport_type().to_string();
        // Removed unused reconnect_attempts_clone

        let target_clone = target.clone();
        
        // Clone references needed for the self-reconnecting task
        let connections_for_task = Arc::clone(&self.connections);
        let reconnect_attempts_for_task = Arc::clone(&self.reconnect_attempts);
        
        let handle = tokio::spawn(async move {
            let mut attempt_count = 0u32;
            let mut current_packet_rx = packet_rx;
            
            loop {
                
                tokio::select! {
                    _ = connection_token_clone.cancelled() => {
                        debug!("Transport task for {} cancelled", target_name_clone);
                        break;
                    }
                    result = handler_clone.start(current_packet_rx, connection_token_clone.child_token(), target_clone.clone(), event_sender.clone()) => {
                        // Both Ok and Err are treated as failures in OSC proxy context
                        // Transport should run forever; if it stops (Ok) or errors (Err), retry
                        match result {
                            Ok(_) => {
                                warn!("Transport handler for {} completed unexpectedly - treating as failure", target_name_clone);
                            },
                            Err(e) => {
                                error!("Transport handler for {} failed (attempt {}): {}", target_name_clone, attempt_count + 1, e);
                            }
                        }
                        
                        // Common failure handling for both Ok and Err cases
                        
                        // Increment attempt count
                        attempt_count += 1;
                        reconnect_attempts_for_task.insert(target_name_clone.clone(), attempt_count);
                        
                        // Calculate retry delay
                        let delay = Self::calculate_reconnect_delay(attempt_count);
                        let next_attempt_at = std::time::SystemTime::now() + delay;
                        
                        // Emit failure event with timing
                        let _ = event_sender.send(ConnectionEvent::Failed {
                            target: target_clone.clone(),
                            next_attempt_at,
                        });
                        
                        // Wait for retry delay
                        if delay > Duration::from_millis(0) {
                            tokio::select! {
                                _ = connection_token_clone.cancelled() => break,
                                _ = tokio::time::sleep(delay) => {}
                            }
                        }
                        
                        // Create new packet channel and transport handler for retry
                        let (new_packet_tx, new_packet_rx) = mpsc::channel(1000);
                        current_packet_rx = new_packet_rx;
                        handler_clone = create_transport_handler(&target_clone.transport);
                        
                        // Update the connection state with new packet sender
                        if let Some(mut connection_ref) = connections_for_task.get_mut(&target_name_clone) {
                            if let ConnectionState::Connected { sender, .. } = connection_ref.value_mut() {
                                *sender = new_packet_tx;
                            }
                        }
                        
                        // Emit reconnecting event and continue loop  
                        let _ = event_sender.send(ConnectionEvent::Reconnecting { target: target_clone.clone() });
                        
                        // Transport handlers will emit Connected events when they successfully establish connections
                    }
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

        // Transport handlers will emit Connected events when they successfully establish connections
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

    pub async fn start_command_listener(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut monitor_handle = self.monitor_handle.write().await;
        if monitor_handle.is_some() {
            return Err("Command listener already started".into());
        }

        let connections = Arc::clone(&self.connections);
        let targets = Arc::clone(&self.targets);
        let main_cancellation_token = self.cancellation_token.clone();
        let event_sender = self.event_sender.clone();
        let reconnect_attempts = self.reconnect_attempts.clone();
        let command_receiver = self.command_receiver.clone();

        let handle = tokio::spawn(async move {
            let mut command_rx = command_receiver.write().await.take().expect("Command receiver should be available");

            loop {
                tokio::select! {
                    _ = main_cancellation_token.cancelled() => {
                        info!("Command listener shutting down");
                        break;
                    }
                    Some(command) = command_rx.recv() => {
                        match command {
                            ConnectionCommand::ManualReconnect { target_name } => {
                                info!("Manual reconnect requested for target: {}", target_name);

                                // Reset attempt counter and cancel current connection
                                reconnect_attempts.insert(target_name.clone(), 0);

                                if let Some(connection_ref) = connections.get(&target_name) {
                                    if let ConnectionState::Connected { cancellation_token, .. } = connection_ref.value() {
                                        cancellation_token.cancel();
                                        // Self-reconnecting transport will handle the restart
                                    }
                                }
                            },
                            ConnectionCommand::Disconnect { target_name } => {
                                info!("Manual disconnect requested for target: {}", target_name);

                                // Cancel the self-reconnecting transport task
                                if let Some((_, connection)) = connections.remove(&target_name) {
                                    if let ConnectionState::Connected { cancellation_token, .. } = connection {
                                        cancellation_token.cancel();
                                    }
                                }

                                connections.insert(target_name.clone(), ConnectionState::Disconnected);

                                if let Some(target_ref) = targets.get(&target_name) {
                                    let _ = event_sender.send(ConnectionEvent::Disconnected { target: target_ref.value().clone() });
                                }
                            }
                        }
                    }
                }
            }
        });

        *monitor_handle = Some(handle);
        Ok(())
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