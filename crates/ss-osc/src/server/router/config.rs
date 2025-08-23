use derivative::Derivative;

#[derive(Derivative, Debug, Clone, PartialEq, Eq)]
#[derivative(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum OscRouterTargetConfigProtocol {
    #[cfg_attr(feature = "serde", serde(rename = "udp"))]
    OscUdp,

    #[cfg_attr(feature = "serde", serde(rename = "tcp"))]
    OscTcp,

    #[derivative(Default)]
    #[cfg_attr(feature = "serde", serde(rename = "websocket"))]
    OscWebsocket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OscRouterTargetConfig {
    // todo: add transport types (e.g., TCP, WebSocket)

    /// The target address to route the packet to.
    to: std::net::SocketAddr,

    #[cfg_attr(feature = "serde", serde(default))]
    protocol: OscRouterTargetConfigProtocol,

    /// Optional address rewrite for the routed packet.
    rewrite: Option<String>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OscRouterRouteConfig {
    /// The OSC address pattern to match.
    #[cfg_attr(feature = "serde", serde(with = "serde_regex"))]
    address: regex::Regex,

    /// Whether to stop propagating packets down the pipeline if matched.
    stop_propagation: bool,

    route: Vec<OscRouterTargetConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_toml() {
        let toml_str = r#"
            [[routes]]
            address = "/example/.*"
            stop_propagation = true

            [[routes.route]]
            to = "127.0.0.1:9000"
            rewrite = "/new/address"
        "#;

        let config: Vec<OscRouterRouteConfig> = toml::from_str(toml_str).unwrap();

        assert_eq!(config.len(), 1);
        assert_eq!(config[0].address.as_str(), "/example/.*");
        assert!(config[0].stop_propagation);
        assert_eq!(config[0].route.len(), 1);
        assert_eq!(config[0].route[0].to, "127.0.0.1:9000".parse().unwrap());
        assert_eq!(config[0].route[0].protocol, OscRouterTargetConfigProtocol::OscUdp);
        assert_eq!(config[0].route[0].rewrite.as_deref(), Some("/new/address"));
    }
}