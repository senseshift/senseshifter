use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{OriginalUri, Query, State};
use axum::http::Uri;
use axum::{Router, routing::any};
use tower_http::trace::TraceLayer;

#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;
#[cfg(feature = "tls")]
use rustls::ServerConfig as TlsServerConfig;

use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use tokio::net::TcpListener;
use tokio_util::future::FutureExt;
use tokio_util::sync::CancellationToken;

use async_trait::async_trait;
use getset::WithSetters;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast, mpsc};
use tracing::*;

mod config;
pub(crate) mod handlers;

use crate::server::{HapticManagerCommand, HapticManagerEvent};
pub use config::*;
pub use handlers::{HandlerBuilder, MessageHandler};

// Cloneable app state for axum router
#[derive(Clone)]
pub struct AppState {
  command_sender: mpsc::Sender<HapticManagerCommand>,
  event_sender: broadcast::Sender<HapticManagerEvent>,
  cancellation_token: CancellationToken,
}

async fn kickstart_ws(socket: &mut WebSocket) -> Result<(), axum::Error> {
  socket
    .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
    .await
}

/// Strategy trait for building handlers with different construction patterns
#[async_trait]
trait HandlerBuildStrategy<H: MessageHandler>: Send + Sync {
  async fn build_handler(
    &self,
    context: H::Context,
    command_tx: mpsc::Sender<HapticManagerCommand>,
    ws_tx: mpsc::UnboundedSender<Message>,
    token: CancellationToken,
  ) -> anyhow::Result<H>;
}

/// Standard strategy: build handler directly using the trait
struct StandardHandlerStrategy;

#[async_trait]
impl<H: MessageHandler> HandlerBuildStrategy<H> for StandardHandlerStrategy {
  async fn build_handler(
    &self,
    context: H::Context,
    command_tx: mpsc::Sender<HapticManagerCommand>,
    ws_tx: mpsc::UnboundedSender<Message>,
    token: CancellationToken,
  ) -> anyhow::Result<H> {
    H::Builder::new(context, command_tx, ws_tx)
      .with_cancellation_token(token)
      .build()
      .await
  }
}

/// V4 Composition strategy: build V3 externally, then wrap in V4
#[cfg(feature = "v4")]
struct V4CompositionStrategy;

#[cfg(feature = "v4")]
#[async_trait]
impl HandlerBuildStrategy<handlers::v4::FeedbackHandler> for V4CompositionStrategy {
  async fn build_handler(
    &self,
    context: handlers::v4::AppContext,
    command_tx: mpsc::Sender<HapticManagerCommand>,
    ws_tx: mpsc::UnboundedSender<Message>,
    token: CancellationToken,
  ) -> anyhow::Result<handlers::v4::FeedbackHandler> {
    // Convert V4 context to V3 context for the wrapped handler
    let v3_context: handlers::v3::AppContext = (&context).into();

    // Create interceptor channel for V3 messages
    let (v3_message_tx, v3_message_rx) = mpsc::unbounded_channel::<Message>();

    // Build V3 handler with interceptor sender
    let v3_handler = handlers::v3::FeedbackHandlerBuilder::new(
      v3_context,
      command_tx.clone(),
      v3_message_tx, // V3 messages will be captured here
    )
    .with_cancellation_token(token.clone())
    .build()
    .await?;

    // Build V4 handler with the V3 handler and interceptor receiver
    handlers::v4::FeedbackHandlerBuilder::new(context, command_tx, ws_tx)
      .with_v3_handler(v3_handler)
      .with_v3_message_receiver(v3_message_rx)
      .with_cancellation_token(token)
      .build()
      .await
  }
}

/// Unified WebSocket upgrade handler with pluggable handler build strategy
async fn upgrade_websocket_with_strategy<
  H: MessageHandler,
  S: HandlerBuildStrategy<H> + Send + 'static,
>(
  ws: WebSocketUpgrade,
  Query(context): Query<H::Context>,
  State(app_state): State<AppState>,
  strategy: S,
) -> axum::response::Response {
  let event_rx = app_state.event_sender.subscribe();

  info!("Received WebSocket upgrade request");

  ws.on_upgrade(async move |mut socket| {
    // Kickstart WebSocket with initial ping
    match kickstart_ws(&mut socket).await {
      Ok(_) => {}
      Err(e) => {
        error!("Failed to kickstart websocket: {}", e);
        return;
      }
    }

    let connection_token = app_state.cancellation_token.child_token();
    let (sender, mut receiver) = socket.split();

    // Create channel for WebSocket message sending (lock-free)
    let (ws_tx, mut ws_rx) = mpsc::unbounded_channel::<Message>();

    // Build handler using the provided strategy
    let mut handler = match strategy
      .build_handler(
        context,
        app_state.command_sender,
        ws_tx.clone(),
        connection_token.clone(),
      )
      .await
    {
      Ok(h) => h,
      Err(e) => {
        error!("Failed to create handler: {}", e);
        return;
      }
    };

    // Call connection opened handler
    if let Err(e) = handler.handle_connection_opened().await {
      error!("Failed to handle connection opened: {}", e);
      return;
    }

    let handler = Arc::new(Mutex::new(handler));

    // Task 1: WebSocket sender (dedicated I/O task)
    let connection_token_sender = connection_token.clone();
    let sender_task = tokio::spawn(async move {
      let mut sender = sender;
      loop {
        tokio::select! {
          Some(msg) = ws_rx.recv() => {
            if let Err(e) = sender.send(msg).await {
              error!("Failed to send WebSocket message: {}", e);
              break;
            }
          }
          _ = connection_token_sender.cancelled() => break,
        }
      }
      debug!("Sender task completed");
    });

    // Task 2: WebSocket receiver
    let handler_receiver = Arc::clone(&handler);
    let connection_token_receiver = connection_token.clone();
    let ws_tx_ping = ws_tx.clone();
    let receiver_task = tokio::spawn(async move {
      loop {
        tokio::select! {
          Some(msg) = receiver.next() => {
            match msg {
              Ok(message) => {
                debug!("Received message: {:?}", message);

                let (result, should_close) = match message {
                  Message::Text(text) => {
                    let res = handler_receiver.lock().await.handle_text_message(&text).await;
                    (res, false)
                  },
                  Message::Binary(data) => {
                    let res = handler_receiver.lock().await.handle_binary_message(&data).await;
                    (res, false)
                  },
                  Message::Ping(data) => {
                    debug!("Received Ping, responding with Pong");
                    let pong_result = ws_tx_ping.send(Message::Pong(data));
                    if let Err(e) = pong_result {
                      error!("Failed to send pong: {}", e);
                    }
                    (Ok(()), false)
                  },
                  Message::Pong(_) => {
                    debug!("Received Pong");
                    (Ok(()), false)
                  },
                  Message::Close(_) => {
                    info!("Received Close, closing connection");
                    let res = handler_receiver.lock().await.handle_close().await;
                    (res, true)
                  },
                };

                if let Err(e) = result {
                  error!("Handler error: {}", e);
                }

                if should_close {
                  break;
                }
              }
              Err(e) => {
                error!("WebSocket error: {:?}", e);
                break;
              }
            }
          }
          _ = connection_token_receiver.cancelled() => break,
        }
      }
      debug!("Receiver task completed");
    });

    // Task 3: Event broadcaster
    let handler_events = Arc::clone(&handler);
    let mut event_rx = event_rx.resubscribe();
    let connection_token_events = connection_token.clone();
    let event_task = tokio::spawn(async move {
      loop {
        tokio::select! {
          Ok(event) = event_rx.recv() => {
            if let Err(e) = handler_events.lock().await.handle_haptic_event(&event).await {
              error!("Broadcast handler error: {}", e);
            }
          }
          _ = connection_token_events.cancelled() => break,
        }
      }
      debug!("Event handler task completed");
    });

    // Wait for any task to complete or cancellation
    tokio::select! {
      _ = sender_task => {},
      _ = receiver_task => {},
      _ = event_task => {},
      _ = connection_token.cancelled() => {},
    }

    info!("WebSocket connection closed gracefully");
  })
}

// Simplified upgrade functions using the strategy pattern

/// Standard WebSocket upgrade handler using the default build strategy
async fn upgrade_websocket_standard<H: MessageHandler>(
  ws: WebSocketUpgrade,
  Query(context): Query<H::Context>,
  State(app_state): State<AppState>,
) -> axum::response::Response {
  upgrade_websocket_with_strategy::<H, StandardHandlerStrategy>(
    ws,
    Query(context),
    State(app_state),
    StandardHandlerStrategy,
  )
  .await
}

/// V4 WebSocket upgrade handler using the composition strategy
#[cfg(feature = "v4")]
async fn upgrade_websocket_v4_composition(
  ws: WebSocketUpgrade,
  Query(context): Query<handlers::v4::AppContext>,
  State(app_state): State<AppState>,
) -> axum::response::Response {
  upgrade_websocket_with_strategy::<handlers::v4::FeedbackHandler, V4CompositionStrategy>(
    ws,
    Query(context),
    State(app_state),
    V4CompositionStrategy,
  )
  .await
}

#[derive(Debug, Clone, WithSetters)]
pub struct BhWebsocketServerBuilder {
  config: BhWebsocketServerConfig,
  command_sender: mpsc::Sender<HapticManagerCommand>,
  event_sender: broadcast::Sender<HapticManagerEvent>,

  #[cfg(feature = "tls")]
  #[getset(set_with = "pub")]
  tls_config: Option<TlsServerConfig>,

  #[getset(set_with = "pub")]
  cancellation_token: Option<CancellationToken>,
}

impl BhWebsocketServerBuilder {
  pub fn new(
    config: BhWebsocketServerConfig,
    command_sender: mpsc::Sender<HapticManagerCommand>,
    event_sender: broadcast::Sender<HapticManagerEvent>,
  ) -> Self {
    Self {
      config,
      command_sender,
      event_sender,

      #[cfg(feature = "tls")]
      tls_config: None,

      cancellation_token: None,
    }
  }

  pub async fn build(self) -> anyhow::Result<()> {
    #[cfg(feature = "tls")]
    if self.config.listen().is_none() && self.config.listen_tls().is_none() {
      return Err(anyhow::anyhow!(
        "Both listen and listen_tls are not set, at least one of them must be set",
      ));
    }

    #[cfg(not(feature = "tls"))]
    if self.config.listen().is_none() {
      return Err(anyhow::anyhow!(
        "Listen is not set, it must be set when TLS feature is disabled",
      ));
    }

    #[cfg(feature = "tls")]
    let tls_config = match self.tls_config {
      Some(tls_config) => Some(RustlsConfig::from_config(Arc::from(tls_config))),
      None => {
        if self.config.tls_cert_path().is_none() || self.config.tls_key_path().is_none() {
          warn!("TLS is not enabled, because cert path or key path is not set");
          None
        } else {
          match RustlsConfig::from_pem_file(
            self.config.tls_cert_path().as_ref().unwrap(),
            self.config.tls_key_path().as_ref().unwrap(),
          )
          .await
          {
            Ok(config) => Some(config),
            Err(err) => {
              error!("Failed to load TLS config, TLS is disabled: {err}");
              None
            }
          }
        }
      }
    };

    let cancellation_token = self.cancellation_token.unwrap_or_default();

    // Create version-specific upgrade handlers using State pattern
    let app_state = AppState {
      command_sender: self.command_sender,
      event_sender: self.event_sender,
      cancellation_token: cancellation_token.clone(),
    };

    let mut app = Router::new();

    // yeah, trailing URLs with slashes are routed separately...

    #[cfg(feature = "v1")]
    {
      app = app
        .route(
          "/feedbacks",
          any(upgrade_websocket_standard::<handlers::v1::FeedbackHandler>),
        )
        .route(
          "/feedbacks/",
          any(upgrade_websocket_standard::<handlers::v1::FeedbackHandler>),
        );
    }

    #[cfg(feature = "v2")]
    {
      app = app
        .route(
          "/v2/feedbacks",
          any(upgrade_websocket_standard::<handlers::v2::FeedbackHandler>),
        )
        .route(
          "/v2/feedbacks/",
          any(upgrade_websocket_standard::<handlers::v2::FeedbackHandler>),
        );
    }

    #[cfg(feature = "v3")]
    {
      app = app
        .route(
          "/v3/feedback",
          any(upgrade_websocket_standard::<handlers::v3::FeedbackHandler>),
        )
        .route(
          "/v3/feedback/",
          any(upgrade_websocket_standard::<handlers::v3::FeedbackHandler>),
        );
    }

    #[cfg(feature = "v4")]
    {
      app = app
        .route("/v4/feedback", any(upgrade_websocket_v4_composition))
        .route("/v4/feedback/", any(upgrade_websocket_v4_composition));
    }

    // log URLs to wrong paths
    let app = app.fallback(any(
      async move |uri: Uri, OriginalUri(original_uri): OriginalUri| {
        warn!("Received request to unknown path: {uri}, original path: {original_uri}",);
      },
    ));
    let app = app
      .with_state(app_state)
      .layer(TraceLayer::new_for_http())
      .into_make_service();

    if let Some(listen) = self.config.listen() {
      let app = app.clone();
      let cancellation_token = cancellation_token.child_token();

      match TcpListener::bind(listen).await {
        Ok(listener) => {
          info!("Started listener on {listen}");

          tokio::spawn(
            async move {
              let result = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                  cancellation_token.cancelled().await;
                })
                .await;

              if let Err(err) = result {
                error!("Server error: {err:?}");
              }

              info!("Server exited gracefully");
            }
            .instrument(info_span!("bh_ws_serve")),
          );
        }
        Err(err) => {
          error!("Failed to bind to {listen}: {err}");
        }
      };
    }

    #[cfg(feature = "tls")]
    if let Some(tls_config) = tls_config
      && let Some(listen_tls) = self.config.listen_tls()
    {
      let app = app.clone();
      let cancellation_token = cancellation_token.child_token();

      info!("Starting TLS listener on {}", listen_tls);

      let mut tls_server = axum_server::bind_rustls(*listen_tls, tls_config);
      tls_server.http_builder().http2().enable_connect_protocol();

      tokio::spawn(
        async move {
          if let Some(Err(err)) = tls_server
            .serve(app)
            .with_cancellation_token(&cancellation_token)
            .await
          {
            error!("TLS server error: {err:?}");
          }

          info!("TLS server exited gracefully");
        }
        .instrument(info_span!("bh_ws_serve_tls")),
      );
    }

    Ok(())
  }
}
