use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use getset::WithSetters;

#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;
#[cfg(feature = "tls")]
use rustls::ServerConfig as TlsServerConfig;

use std::sync::Arc;

use tracing::*;

mod config;

pub use config::*;

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
              error!("Failed to load TLS config, TLS is disabled: {}", err);
              None
            }
          }
        }
      }
    };

    let cancellation_token = self.cancellation_token.unwrap_or_default();

    let app = Router::<()>::new();

    #[cfg(feature = "v1")]
    let app = app.route(
      "/feedbacks",
      get(|| async {
        info!("v1 feedback endpoint hit");
      }),
    );

    #[cfg(feature = "v2")]
    let app = app.route(
      "/v2/feedbacks",
      get(|| async {
        info!("v2 feedback endpoint hit");
      }),
    );

    #[cfg(feature = "v3")]
    let app = app.route(
      "/v3/feedback",
      get(|| async {
        info!("v3 feedback endpoint hit");
      }),
    );

    #[cfg(feature = "v4")]
    let app = app.route(
      "/v4/feedback",
      get(|| async {
        info!("v4 feedback endpoint hit");
      }),
    );

    // let app = app.layer((
    //   TraceLayer::new_for_http(),
    //   // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
    //   // requests don't hang forever.
    //   TimeoutLayer::new(Duration::from_secs(10)),
    // ));

    if let Some(listen) = self.config.listen() {
      let app = app.clone();
      let cancellation_token = cancellation_token.child_token();

      match TcpListener::bind(listen).await {
        Ok(listener) => {
          info!("Started listener on {}", listen);

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
          error!("Failed to bind to {}: {}", listen, err);
        }
      };
    }

    #[cfg(feature = "tls")]
    if let Some(tls_config) = tls_config
      && let Some(listen_tls) = self.config.listen_tls()
    {
      info!("Starting TLS listener on {}", listen_tls);

      let mut tls_server = axum_server::bind_rustls(*listen_tls, tls_config);

      tls_server.http_builder().http2().enable_connect_protocol();

      let app = app.clone();

      tokio::spawn(
        async move {
          if let Err(err) = tls_server.serve(app.into_make_service()).await {
            error!("TLS server error: {}", err);
          }

          info!("TLS server existed gracefully");
        }
        .instrument(info_span!("bh_ws_server_tls")),
      );
    }

    Ok(())
  }
}
