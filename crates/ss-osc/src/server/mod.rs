use std::net::SocketAddr;
use futures::pin_mut;
use futures_util::StreamExt;
use crate::Result;
use crate::server::connection_manager::{ConnectionManager, Target};
use crate::server::router::{OscRouterRouteForwardRuntime, OscRouterRouteRuntime};
use tokio::sync::{broadcast};
use tokio_stream::wrappers::BroadcastStream;

pub mod config;
pub mod router;
pub mod connection_manager;
mod task;

#[derive(Debug, Clone)]
pub enum OscServerEvent {
    InboundPacket {
        packet: rosc::OscPacket,
        from: SocketAddr,
    },
    ConnectionManagerEvent(connection_manager::ConnectionEvent),
}

pub struct OscServerBuilder {
    config: config::OscServerConfig,
    cancel_token: Option<tokio_util::sync::CancellationToken>,
}

impl OscServerBuilder {
    pub fn new(
        config: config::OscServerConfig,
    ) -> Self {
        Self {
            config,
            cancel_token: None,
        }
    }

    /// Add an optional cancellation token that will be used to control the server task
    pub fn with_cancel_token(mut self, token: tokio_util::sync::CancellationToken) -> Self {
        self.cancel_token = Some(token);
        self
    }
}

impl OscServerBuilder {
    pub fn build(self) -> Result<OscServer> {
        let connection_manager = ConnectionManager::new();

        let mut targets: Vec<Target> = Vec::new();
        let mut router_routes: Vec<OscRouterRouteRuntime> = Vec::new();

        for route in self.config.routes() {
            let mut runtime = OscRouterRouteRuntime {
                address: route.address.clone(),
                stop_propagation: route.stop_propagation,
                forward: vec![],
            };

            for forward in route.forward().iter() {
                let target_name = format!("{} ({})", forward.target.remote_address(), forward.target.transport_type());

                let target = Target {
                    name: target_name.clone(),
                    transport: forward.target.clone().into(),
                };

                targets.push(target);
                runtime.forward.push(OscRouterRouteForwardRuntime {
                    to: target_name,
                    rewrite: forward.rewrite.clone(),
                });
            }

            router_routes.push(runtime);
        }

        // Add targets to connection manager
        let forward_targets = connection_manager.add_targets(targets)?;

        let router = router::OscRouter::new(
            router_routes,
            forward_targets,
        );

        let cancellation_token = self.cancel_token.unwrap_or_else(|| tokio_util::sync::CancellationToken::new());
        let (tx, rx) = tokio::sync::broadcast::channel(10);
        let connection_manager_event_receiver = connection_manager.subscribe_to_events();

        let task = task::OscServerTask::new(
            router,
            connection_manager,
            self.config.server.udp,
            self.config.server.tcp,
            cancellation_token.clone(),
            tx.clone(),
        );

        let _join_token = tokio::spawn(
            async move {
                pin_mut!(task);

                if let Err(err) = task.run().await {
                    tracing::error!("OSC Server Task failed: {:?}", err);
                }

                tracing::info!("OSC Server Task exited.");
            }
        );

        Ok(OscServer {
            cancel_token: cancellation_token,
            event_receiver: rx,
            connection_manager_event_receiver,
        })
    }
}

pub struct OscServer {
    cancel_token: tokio_util::sync::CancellationToken,
    event_receiver: tokio::sync::broadcast::Receiver<OscServerEvent>,
    connection_manager_event_receiver: tokio::sync::broadcast::Receiver<connection_manager::ConnectionEvent>,
}

impl OscServer {
    pub fn subscribe_to_events(&self) -> (tokio::sync::broadcast::Receiver<OscServerEvent>, tokio::sync::broadcast::Receiver<connection_manager::ConnectionEvent>) {
        (self.event_receiver.resubscribe(), self.connection_manager_event_receiver.resubscribe())
    }

    pub async fn shutdown(&self) {
        self.cancel_token.cancel();
    }
}