use derivative::Derivative;
use getset::{Getters, WithSetters};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

#[derive(Derivative, Debug, Clone, Getters, WithSetters)]
#[getset(get = "pub")]
pub struct BhWebsocketServerConfig {
  #[getset(set_with = "pub")]
  listen: Option<SocketAddr>,

  #[cfg(feature = "tls")]
  #[getset(set_with = "pub")]
  listen_tls: Option<SocketAddr>,

  #[cfg(feature = "tls")]
  #[getset(set_with = "pub")]
  tls_cert_path: Option<PathBuf>,

  #[cfg(feature = "tls")]
  #[getset(set_with = "pub")]
  tls_key_path: Option<PathBuf>,
}

impl Default for BhWebsocketServerConfig {
  fn default() -> Self {
    BhWebsocketServerConfig {
      listen: Some(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 15881)),

      #[cfg(feature = "tls")]
      listen_tls: Some(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 15882)),

      #[cfg(feature = "tls")]
      tls_cert_path: Some(PathBuf::from("./certs/cert.pem")),

      #[cfg(feature = "tls")]
      tls_key_path: Some(PathBuf::from("./certs/key.pem")),
    }
  }
}
