use axum::Router;

#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;

use axum::routing::get;
use getset::WithSetters;

#[cfg(feature = "tls")]
use rustls::ServerConfig as TlsServerConfig;
use std::sync::Arc;
use tracing::*;

mod config;

pub use config::*;

#[derive(Debug, Clone, WithSetters)]
pub struct BhWebsocketServerBuilder {
  config: config::BhWebsocketServerConfig,

  #[cfg(feature = "tls")]
  #[getset(set_with = "pub")]
  tls_config: Option<TlsServerConfig>,
}

impl BhWebsocketServerBuilder {
  pub fn new(config: config::BhWebsocketServerConfig) -> Self {
    Self {
      config,

      #[cfg(feature = "tls")]
      tls_config: None,
    }
  }

  pub async fn build(self) -> anyhow::Result<()> {
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

    if let Some(listen) = self.config.listen() {
      info!("Starting listener on {}", listen);

      let mut server = axum_server::bind(*listen);

      server.http_builder().http2().enable_connect_protocol();

      let app = app.clone();

      tokio::spawn(
        async move {
          if let Err(err) = server.serve(app.into_make_service()).await {
            error!("TLS server error: {}", err);
          }

          info!("TLS server existed gracefully");
        }
        .instrument(info_span!("bh_ws_server")),
      );
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
