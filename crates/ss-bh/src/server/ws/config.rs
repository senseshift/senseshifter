use derivative::Derivative;
use getset::Getters;
use std::net::{Ipv4Addr, SocketAddr};

#[derive(Derivative, Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct BhWebsocketServerConfig {
  listen: Option<SocketAddr>,

  #[cfg(feature = "tls")]
  listen_tls: Option<SocketAddr>,

  #[cfg(feature = "tls")]
  tls_cert_path: Option<String>,

  #[cfg(feature = "tls")]
  tls_key_path: Option<String>,
}

impl Default for BhWebsocketServerConfig {
  fn default() -> Self {
    BhWebsocketServerConfig {
      listen: Some(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 15881)),

      #[cfg(feature = "tls")]
      listen_tls: Some(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 15882)),

      #[cfg(feature = "tls")]
      tls_cert_path: Some("./certs/cert.pem".to_string()),

      #[cfg(feature = "tls")]
      tls_key_path: Some("./certs/key.pem".to_string()),
    }
  }
}
