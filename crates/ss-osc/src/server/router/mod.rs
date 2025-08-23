pub mod config;

use std::collections::HashMap;
use std::net::SocketAddr;
use rosc::{OscMessage, OscPacket};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

// Re-export for convenience
pub use config::{RouterRouteConfig as OscRouterRoute, RouterForwardConfig as OscRouterRouteForward};

// Runtime router types that reference targets by name
#[derive(Debug)]
pub struct OscRouterRouteForwardRuntime {
    pub to: String, // Key to look up in forward_targets
    pub rewrite: Option<String>,
}

#[derive(Debug)]
pub struct OscRouterRouteRuntime {
    pub address: regex::Regex,
    pub stop_propagation: bool,
    pub forward: Vec<OscRouterRouteForwardRuntime>,
}

#[derive(Debug)]
pub struct OscRouter {
    routes: Vec<OscRouterRouteRuntime>,

    /// Pre-Connected proxy targets
    forward_targets: HashMap<String, mpsc::Sender<OscPacket>>,
    // todo: ^ support multiple transport types (e.g., TCP, WebSocket)
}

impl OscRouter {
    pub fn new(
        routes: Vec<OscRouterRouteRuntime>,
        forward_targets: HashMap<String, mpsc::Sender<OscPacket>>,
    ) -> Self {
        info!(
            "Creating OSC router with {} routes and {} forward targets",
            routes.len(),
            forward_targets.len()
        );
        debug!("Forward targets: {:?}", forward_targets.keys().collect::<Vec<_>>());
        
        Self {
            routes,
            forward_targets,
        }
    }

    fn get_forward_target(&self, name: &str) -> Option<&mpsc::Sender<OscPacket>> {
        self.forward_targets.get(name)
    }
}

impl OscRouter {
    pub async fn route(&self, packet: &OscPacket, from: &SocketAddr) {
        debug!("Routing OSC packet from {}", from);
        self.route_packet(packet, from, 255).await;
    }

    async fn route_packet(
        &self,
        packet: &OscPacket,
        from: &SocketAddr,
        depth: usize
    ) {
        match packet {
            OscPacket::Message(msg) => {
                self.route_message(msg, from).await;
            }
            OscPacket::Bundle(bundle) => {
                for p in &bundle.content {
                    if depth > 0 {
                        Box::pin(self.route_packet(p, from, depth - 1)).await;
                    } else {
                        warn!(
                            "Max bundle depth reached (255), skipping further routing for packet from {}",
                            from
                        );
                    }
                }
            }
        }
    }

    async fn route_message(&self, msg: &OscMessage, _from: &SocketAddr) {
        debug!(
            "Routing OSC message: {} with {} args",
            msg.addr,
            msg.args.len()
        );
        
        let mut matched_routes = 0;
        
        for route in &self.routes {
            // Check if the message address matches the route's address pattern
            let captures = match route.address.captures(&msg.addr) {
                Some(c) => {
                    debug!("Route matched: {} -> pattern {}", msg.addr, route.address.as_str());
                    c
                },
                None => continue,
            };
            
            matched_routes += 1;

            // Route matched, process forwards
            for forward in &route.forward {
                // Get forward target
                let target = match self.get_forward_target(&forward.to) {
                    Some(t) => t,
                    None => {
                        error!(
                            "No forward target '{}' found for route. Available targets: {:?}",
                            forward.to,
                            self.forward_targets.keys().collect::<Vec<_>>()
                        );
                        continue;
                    }
                };

                let mut msg_to_send = msg.clone();

                // Rewrite address if needed
                if let Some(rewrite) = &forward.rewrite {
                    let original_addr = msg_to_send.addr.clone();
                    let new_addr = self.rewrite_address(rewrite, &captures, route.address.capture_names());
                    msg_to_send.addr = new_addr.clone();
                    debug!(
                        "Address rewritten: '{}' -> '{}' using pattern '{}'",
                        original_addr,
                        new_addr,
                        rewrite
                    );
                }

                let message_addr = msg_to_send.addr.clone();
                let packet_to_send = OscPacket::Message(msg_to_send);

                // Send the packet to the target
                match target.send(packet_to_send).await {
                    Ok(_) => {
                        debug!("Successfully forwarded message to target '{}': {}", forward.to, message_addr);
                    },
                    Err(e) => {
                        error!(
                            "Failed to send packet to target '{}': {}. Message: {}",
                            forward.to,
                            e,
                            message_addr
                        );
                    }
                }
            }

            // If stop_propagation is set, break after the first match
            if route.stop_propagation {
                debug!(
                    "Stop propagation enabled for route '{}', stopping further routing",
                    route.address.as_str()
                );
                break;
            }
        }
        
        if matched_routes == 0 {
            debug!("No routes matched for OSC message: {}", msg.addr);
        } else {
            debug!("Message '{}' matched {} route(s)", msg.addr, matched_routes);
        }
    }

    fn rewrite_address(
        &self,
        rewrite: &str,
        captures: &regex::Captures,
        capture_names: regex::CaptureNames,
    ) -> String {
        let mut new_addr = rewrite.to_string();
        let original_pattern = new_addr.clone();

        // Replace numbered captures ($1, $2, etc.)
        let mut numbered_replacements = Vec::new();
        for (i, cap) in captures.iter().enumerate() {
            if let Some(m) = cap {
                let placeholder = format!("${}", i);
                if new_addr.contains(&placeholder) {
                    new_addr = new_addr.replace(&placeholder, m.as_str());
                    numbered_replacements.push((placeholder, m.as_str().to_string()));
                }
            }
        }

        // Replace named captures ($name)
        let mut named_replacements = Vec::new();
        for name in capture_names.flatten() {
            if let Some(m) = captures.name(name) {
                let placeholder = format!("${}", name);
                if new_addr.contains(&placeholder) {
                    new_addr = new_addr.replace(&placeholder, m.as_str());
                    named_replacements.push((placeholder, m.as_str().to_string()));
                }
            }
        }

        // Log replacement details at trace level for debugging
        if !numbered_replacements.is_empty() || !named_replacements.is_empty() {
            debug!(
                "Address rewrite details: '{}' -> '{}'. Numbered: {:?}, Named: {:?}",
                original_pattern,
                new_addr,
                numbered_replacements,
                named_replacements
            );
        }

        new_addr
    }
}

#[cfg(test)]
mod tests {
    use super::OscRouter;
    use rosc::OscMessage;
    use regex::Regex;

    #[test]
    fn test_rewrite_address_no_captures() {
        let router = OscRouter::new(vec![], std::collections::HashMap::new());

        let msg = OscMessage {
            addr: "/foo/bar".to_string(),
            args: vec![],
        };

        let rewrite = "/new/address";
        let regex = Regex::new(r"^/foo/bar$").unwrap();
        let captures = regex.captures(&msg.addr).unwrap();

        let new_addr = router.rewrite_address(rewrite, &captures, regex.capture_names());
        assert_eq!(new_addr, "/new/address");
    }

    #[test]
    fn test_rewrite_address_numbered() {

        let router = OscRouter::new(vec![], std::collections::HashMap::new());

        let msg = OscMessage {
            addr: "/foo/bar/baz".to_string(),
            args: vec![],
        };

        let rewrite = "/new/$1/$2";
        let regex = Regex::new(r"^/foo/(.*)/(.*)$").unwrap();
        let captures = regex.captures(&msg.addr).unwrap();

        let new_addr = router.rewrite_address(rewrite, &captures, regex.capture_names());
        assert_eq!(new_addr, "/new/bar/baz");
    }

    #[test]
    fn test_rewrite_address_named_groups() {
        let router = OscRouter::new(vec![], std::collections::HashMap::new());

        let msg = OscMessage {
            addr: "/foo/bar/baz".to_string(),
            args: vec![],
        };

        let rewrite = "/new/$name1/$name2";
        let regex = Regex::new(r"^/foo/(?P<name1>.*)/(?P<name2>.*)$").unwrap();
        let captures = regex.captures(&msg.addr).unwrap();

        let new_addr = router.rewrite_address(rewrite, &captures, regex.capture_names());
        assert_eq!(new_addr, "/new/bar/baz");
    }

    #[test]
    fn test_rewrite_address_nested_named_groups() {
        let router = OscRouter::new(vec![], std::collections::HashMap::new());

        let regex = Regex::new(r"^/avatar/parameters/(?P<parameter>test_(?P<name1>.*)_(?P<name2>.*))$").unwrap();

        let msg = OscMessage {
            addr: "/avatar/parameters/test_bar_baz".to_string(),
            args: vec![],
        };

        let captures = regex.captures(&msg.addr).unwrap();

        let rewrite = "/new/$parameter/$name1/$name2";

        let new_addr = router.rewrite_address(rewrite, &captures, regex.capture_names());

        assert_eq!(new_addr, "/new/test_bar_baz/bar/baz");
    }

    #[tokio::test]
    async fn test_route() {
        let (tx_foo, mut rx_foo) = tokio::sync::mpsc::channel(1);
        let (tx_baz, mut rx_baz) = tokio::sync::mpsc::channel(1);
        let (tx_catchall, mut rx_catchall) = tokio::sync::mpsc::channel(10);

        let forward_targets = {
            let mut map = std::collections::HashMap::new();

            map.insert("foo".to_string(), tx_foo);
            map.insert("baz".to_string(), tx_baz);
            map.insert("catchall".to_string(), tx_catchall);

            map
        };

        let routes = vec![
            super::OscRouterRouteRuntime {
                address: Regex::new(r"^/foo/(.*)$").unwrap(),
                stop_propagation: false,
                forward: vec![
                    super::OscRouterRouteForwardRuntime {
                        to: "foo".to_string(),
                        rewrite: Some("/bar/$1".to_string()),
                    }
                ],
            },
            super::OscRouterRouteRuntime {
                address: Regex::new(r"^/baz/(.*)$").unwrap(),
                stop_propagation: true,
                forward: vec![
                    super::OscRouterRouteForwardRuntime {
                        to: "baz".to_string(),
                        rewrite: Some("/qux".to_string()),
                    }
                ],
            },
            super::OscRouterRouteRuntime {
                address: Regex::new(r"(.*)").unwrap(),
                stop_propagation: false,
                forward: vec![
                    super::OscRouterRouteForwardRuntime {
                        to: "catchall".to_string(),
                        rewrite: None,
                    }
                ],
            },
        ];

        let router = OscRouter::new(routes, forward_targets);

        let from: std::net::SocketAddr = "127.0.0.1:9000".parse().unwrap();

        let packet_foo = rosc::OscPacket::Message(OscMessage {
            addr: "/foo/test".to_string(),
            args: vec![],
        });

        let packet_baz = rosc::OscPacket::Message(OscMessage {
            addr: "/baz/qux".to_string(),
            args: vec![],
        });

        router.route(&packet_foo, &from).await;
        router.route(&packet_baz, &from).await;

        // `foo/test` should be rewritten to `/bar/test` and sent to `tx_foo`
        let received = rx_foo.recv().await.unwrap();
        if let rosc::OscPacket::Message(msg) = received {
            assert_eq!(msg.addr, "/bar/test");
        } else {
            panic!("Expected OscMessage");
        }

        // `baz/qux` should be rewritten to `/qux` and sent to `tx_baz`
        let received = rx_baz.recv().await.unwrap();
        if let rosc::OscPacket::Message(msg) = received {
            assert_eq!(msg.addr, "/qux");
        } else {
            panic!("Expected OscMessage");
        }

        // Only the `/foo/test` message should be caught by the catchall without rewriting
        let received = rx_catchall.recv().await.unwrap();
        if let rosc::OscPacket::Message(msg) = received {
            assert_eq!(msg.addr, "/foo/test");
        } else {
            panic!("Expected OscMessage");
        }

        // No more messages should be in the catchall
        assert!(rx_catchall.try_recv().is_err());
    }
}