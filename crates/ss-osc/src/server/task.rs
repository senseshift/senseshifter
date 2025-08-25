use crate::Result;

use tokio_util::sync::CancellationToken;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast};
use std::net::{
    SocketAddr,
};
use tracing::{debug, error, info, warn};

use crate::server::router::OscRouter;
use crate::server::connection_manager::ConnectionManager;
use crate::server::OscServerEvent;

#[derive(Debug)]
pub struct OscServerTask {
    router: OscRouter,
    connection_manager: ConnectionManager,

    udp_addrs: Vec<SocketAddr>,
    tcp_addrs: Vec<SocketAddr>,

    cancellation_token: CancellationToken,
    event_sender: broadcast::Sender<OscServerEvent>,
}

impl OscServerTask {
    pub fn new(
        router: OscRouter,
        connection_manager: ConnectionManager,
        udp_addrs: Vec<SocketAddr>,
        tcp_addrs: Vec<SocketAddr>,
        cancellation_token: CancellationToken,
        event_sender: broadcast::Sender<OscServerEvent>,
    ) -> Self {
        if udp_addrs.is_empty() && tcp_addrs.is_empty() {
            panic!("At least one UDP or TCP address must be provided for the OSC server.");
        }

        if tcp_addrs.len() > 0 {
            warn!("TCP transport is configured but not yet fully implemented");
        }

        info!(
            "Creating OSC server task with {} UDP addresses and {} TCP addresses", 
            udp_addrs.len(), 
            tcp_addrs.len()
        );

        Self {
            router,
            connection_manager,
            udp_addrs,
            tcp_addrs,
            cancellation_token,
            event_sender,
        }
    }

    pub async fn run(
        &mut self,
    ) -> Result<()> {
        if self.udp_addrs.is_empty() && self.tcp_addrs.is_empty() {
            return Err(anyhow::anyhow!("At least one UDP or TCP address must be provided for the OSC server."));
        }

        // Start the connection manager
        info!("Starting connection manager...");
        self.connection_manager.connect_all().await.map_err(|e| {
            error!("Failed to connect to targets: {}", e);
            anyhow::anyhow!("Failed to connect to targets: {}", e)
        })?;
        
        // Start command listener for manual reconnect/disconnect
        self.connection_manager.start_command_listener().await.map_err(|e| {
            error!("Failed to start command listener: {}", e);
            anyhow::anyhow!("Failed to start command listener: {}", e)
        })?;
        
        info!("Connection manager started successfully");

        let mut udp_socket = None;

        if self.udp_addrs.len() > 0 {
            udp_socket = Some(UdpSocket::bind(&self.udp_addrs[..]).await?);
            info!("OSC Server listening on UDP: {:?}", udp_socket.as_ref().unwrap().local_addr()?);
        }

        if self.tcp_addrs.len() > 0 {
            // todo: bind TCP sockets
            warn!("TCP transport binding not yet implemented, skipping TCP addresses");
        }

        let mut buf = [0u8; rosc::decoder::MTU];
        info!("OSC server started, ready to receive packets");

        loop {
            tokio::select! {
                result = udp_socket.as_ref().unwrap().recv_from(&mut buf), if udp_socket.is_some() => {
                    match result {
                        Ok((size, addr)) => {
                            self.handle_udp_packet(&buf[..size], &addr).await;
                        },
                        Err(e) => {
                            self.handle_error(anyhow::Error::new(e).context("Error receiving UDP packet")).await;
                        }
                    }
                },

                _ = self.cancellation_token.cancelled() => {
                    info!("OSC server shutdown requested");
                    break;
                },
            }
        }

        // Graceful shutdown
        info!("Shutting down connection manager...");
        self.connection_manager.shutdown().await;
        info!("OSC server shutdown complete");

        Ok(())
    }

    async fn handle_udp_packet(&self, msg: &[u8], from: &SocketAddr) {
        // todo: add ACLs for filtering incoming packets
        debug!("Received {} bytes from {}", msg.len(), from);

        match rosc::decoder::decode_udp(msg) {
            Ok((_, packet)) => {
                debug!("Successfully decoded OSC packet from {}: {:?}", from, packet);

                match self.handle_osc_packet(&packet, from).await {
                    Ok(_) => {
                        debug!("Successfully processed OSC packet from {}", from);
                    },
                    Err(e) => {
                        self.handle_error(e).await;
                    }
                }
            },
            Err(e) => {
                self.handle_error(anyhow::Error::new(e).context("Error decoding OSC packet")).await;
            }
        }
    }

    async fn handle_osc_packet(&self, packet: &rosc::OscPacket, from: &SocketAddr) -> Result<()> {
        // First, route the packet through the router to forward to targets
        debug!("Routing OSC packet through router...");
        self.router.route(packet, from).await;
        debug!("Packet routing completed");

        // Then emit the event for any listeners
        let event = OscServerEvent::InboundPacket {
            packet: packet.clone(),
            from: *from,
        };

        match self.event_sender.send(event) {
            Ok(receiver_count) => {
                debug!("OSC server event sent to {} receivers", receiver_count);
                Ok(())
            },
            Err(e) => {
                warn!("No receivers for OSC server event: {}", e);
                // Don't treat this as an error since it's normal if no one is listening
                Ok(())
            }
        }
    }

    async fn handle_error(&self, error: anyhow::Error) {
        // todo: improve error handling (e.g., report to Sentry, structured metrics, etc.)
        
        error!("OSC Server error: {}", error);
        
        // Log the full error chain for debugging
        let mut source = error.source();
        while let Some(err) = source {
            debug!("Caused by: {}", err);
            source = err.source();
        }
    }
}