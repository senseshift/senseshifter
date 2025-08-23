use derivative::Derivative;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OscRouterForwardTargetOscUdpConfig {
    to: std::net::SocketAddr,
    from: std::net::SocketAddr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OscRouterForwardTargetOscTcpConfig {
    to: std::net::SocketAddr,
    // from: std::net::SocketAddr,
}


#[derive(Derivative, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "lowercase"))]
pub enum OscRouterForwardTargetConfig {
    #[cfg_attr(feature = "serde", serde(rename = "udp"))]
    OscUdp(OscRouterForwardTargetOscUdpConfig),

    #[cfg_attr(feature = "serde", serde(rename = "tcp"))]
    OscTcp(OscRouterForwardTargetOscTcpConfig),

    // todo: add WebSocket, etc.
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OscRouterForwardConfig {
    /// The target address to route the packet to.
    #[cfg_attr(feature = "serde", serde(flatten))]
    target: OscRouterForwardTargetConfig,

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

    #[cfg_attr(feature = "serde", serde(default))]
    forward: Vec<OscRouterForwardConfig>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OscRouterConfig {
    routes: Vec<OscRouterRouteConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_yaml() {
        let yaml_str = r#"
            routes:
              - address: /avatar/parameters/(?<parameter>bHaptics_(?<position>.+)_(?<index>\d+)_bool)
                stop_propagation: false
                forward:
                  - type: udp
                    to: 127.0.0.1:23456 # target socket
                    from: 127.0.0.1:23457 # source host socket for UDP
                    rewrite: /$position/$index/bool

                  - type: tcp
                    to: 127.0.0.1:34567

                  # - type: websocket
                  #   to: ws://127.0.0.1:45678/osc

              - address: /avatar/parameters/bHaptics_(?<parameter>.+)
                stop_propagation: true

              - address: /avatar/parameters/(?<parameter>.+)
                stop_propagation: false
        "#;

        let config: OscRouterConfig = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(config.routes.len(), 3);
        assert_eq!(config.routes[0].address.as_str(), "/avatar/parameters/(?<parameter>bHaptics_(?<position>.+)_(?<index>\\d+)_bool)");
        assert!(!config.routes[0].stop_propagation);
        assert_eq!(config.routes[0].forward.len(), 2);
        assert_eq!(config.routes[0].forward[0].target, OscRouterForwardTargetConfig::OscUdp(OscRouterForwardTargetOscUdpConfig {
            to: "127.0.0.1:23456".parse().unwrap(),
            from: "127.0.0.1:23457".parse().unwrap(),
        }));
        assert_eq!(config.routes[0].forward[0].rewrite.as_deref(), Some("/$position/$index/bool"));
        assert_eq!(config.routes[0].forward[1].target, OscRouterForwardTargetConfig::OscTcp(OscRouterForwardTargetOscTcpConfig {
            to: "127.0.0.1:34567".parse().unwrap(),
        }));
        assert!(config.routes[0].forward[1].rewrite.is_none());

        assert_eq!(config.routes[1].address.as_str(), "/avatar/parameters/bHaptics_(?<parameter>.+)");
        assert!(config.routes[1].stop_propagation);
        assert_eq!(config.routes[1].forward.len(), 0);

        assert_eq!(config.routes[2].address.as_str(), "/avatar/parameters/(?<parameter>.+)");
        assert!(!config.routes[2].stop_propagation);
        assert_eq!(config.routes[2].forward.len(), 0);
    }
}