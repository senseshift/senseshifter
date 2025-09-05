use getset::Getters;
use std::net::SocketAddr;

#[derive(Debug, Clone, Getters)]
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
