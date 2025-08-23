#[derive(Debug)]
pub struct OscRouterTargetConfig {
    // todo: add transport types (e.g., TCP, WebSocket)

    /// The target address to route the packet to.
    to: std::net::SocketAddr,

    rewrite: Option<String>,
}

#[derive(Debug)]
pub struct OscRouterRouteConfig {
    /// The OSC address pattern to match.
    address: regex::Regex,

    /// Whether to stop propagating packets down the pipeline if matched.
    stop_propagation: bool,

    route: Vec<OscRouterTargetConfig>,
}
