use axum::body::Bytes;
use axum::extract::OriginalUri;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::Uri;
use axum::{Router, routing::any};
use tower_http::trace::TraceLayer;

#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;
#[cfg(feature = "tls")]
use rustls::ServerConfig as TlsServerConfig;

use futures_util::stream::StreamExt;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use futures_util::SinkExt;
use getset::WithSetters;
use std::sync::Arc;
use tokio_util::future::FutureExt;
use tracing::*;

mod config;
mod handlers;

pub use config::*;

async fn kickstart_ws(socket: &mut WebSocket) -> Result<(), axum::Error> {
  socket
    .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
    .await
}

async fn upgrade_websocket(
  ws: WebSocketUpgrade,
  // Query(_app_ctx): Query<handlers::v3::AppContext>,
) -> axum::response::Response {
  info!("Received V3 SDK request");

  ws.on_upgrade(async move |mut socket| {
    match kickstart_ws(&mut socket).await {
      Ok(_) => {}
      Err(e) => {
        error!("Failed to kickstart websocket: {}", e);
        return;
      }
    }

    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(async move {
      loop {
        tokio::select! {
          Some(msg) = receiver.next() => {
            match msg {
              Ok(message) => {
                debug!("Received message: {:?}", message);

                match message {
                  Message::Text(_) | Message::Binary(_) => {
                    let raw_msg = message.into_text();

                    info!("Received Text/Binary message: {:?}", raw_msg);
                  },
                  Message::Ping(bytes) => {
                    debug!("Received Ping: {:?}", bytes);
                    if let Err(e) = sender.send(Message::Pong(bytes)).await {
                      error!("Failed to send Pong: {}", e);
                    }
                  },
                  Message::Pong(_) => {
                    debug!("Received Pong");
                  },
                  Message::Close(_) => {
                    info!("Received Close, closing connection");
                    break;
                  },
                }
              }
              Err(e) => {
                error!("WebSocket error: {:?}", e);
                break; // Connection error, exit the loop
              }
            }
          }
        }
      }

      info!("WebSocket connection closed");
    });
  })
}

#[derive(Debug, Clone, WithSetters)]
pub struct BhWebsocketServerBuilder {
  config: BhWebsocketServerConfig,

  #[cfg(feature = "tls")]
  #[getset(set_with = "pub")]
  tls_config: Option<TlsServerConfig>,

  #[getset(set_with = "pub")]
  cancellation_token: Option<CancellationToken>,
}

impl BhWebsocketServerBuilder {
  pub fn new(config: BhWebsocketServerConfig) -> Self {
    Self {
      config,

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

    let app = Router::<()>::new();

    #[cfg(feature = "v1")]
    let app = app
      .route("/feedbacks", any(upgrade_websocket))
      .route("/feedbacks/", any(upgrade_websocket));

    #[cfg(feature = "v2")]
    let app = app
      .route("/v2/feedbacks", any(upgrade_websocket))
      .route("/v2/feedbacks/", any(upgrade_websocket));

    #[cfg(feature = "v3")]
    let app = app
      .route("/v3/feedback", any(upgrade_websocket))
      .route("/v3/feedback/", any(upgrade_websocket));

    #[cfg(feature = "v4")]
    let app = app
      .route("/v4/feedback", any(upgrade_websocket))
      .route("/v4/feedback/", any(upgrade_websocket));

    // log URLs to wrong paths
    let app = app.fallback(any(
      async move |uri: Uri, OriginalUri(original_uri): OriginalUri| {
        warn!("Received request to unknown path: {uri}, original path: {original_uri}",);
      },
    ));
    let app = app.layer(TraceLayer::new_for_http());
    let app = app.into_make_service();

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
                error!("TLS server error: {}", err);
              }

              info!("TLS server existed gracefully");
            }
            .instrument(info_span!("bh_ws_server")),
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
            error!("TLS server error: {err}");
          }

          info!("TLS server existed gracefully");
        }
        .instrument(info_span!("bh_ws_server_tls")),
      );
    }

    Ok(())
  }
}
