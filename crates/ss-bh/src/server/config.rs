use std::net::{Ipv4Addr, SocketAddr};
use derivative::Derivative;
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Derivative, Serialize, Deserialize, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq, Default)]
pub struct BhServerConfig {
    #[getset(get = "pub")]
    #[derivative(Default(value = "default_listen()"))]
    #[serde(default = "default_listen")]
    listen: Vec<SocketAddr>,
}

fn default_listen() -> Vec<SocketAddr> {
    vec![SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 15881)]
}