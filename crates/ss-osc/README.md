# SS-OSC: OSC Connection Management and Routing

This crate provides OSC (Open Sound Control) connection management and routing functionality for the SenseShifter project.

## Features

- **Connection Management**: Automatically connects to OSC targets (UDP/TCP) and maintains connections
- **Auto-Reconnection**: Automatically reconnects to failed targets with configurable intervals
- **Routing**: Route OSC messages between targets with pattern matching and address rewriting
- **Graceful Shutdown**: Uses hierarchical cancellation tokens for clean shutdown of all connections
- **Individual Connection Control**: Each connection can be cancelled independently using child tokens
- **Persistent Routing**: Router connections remain valid across reconnections via proxy senders

## Applications

This crate is used by:

- **[OSC Proxy](../../apps/osc-proxy/README.md)**: Terminal-based OSC routing proxy with real-time monitoring
- **SenseShifter**: Main application for haptic feedback routing

For configuration examples and user documentation, see the [OSC Proxy README](../../apps/osc-proxy/README.md).

## Architecture

### Connection Manager (`server::connection_manager`)

The `ConnectionManager` handles:
- Managing multiple OSC targets (UDP/TCP)
- Establishing and maintaining connections
- Auto-reconnecting failed connections with logarithmic backoff
- Providing MPSC channels for the router
- Event broadcasting for connection status updates

### Router (`server::router`)

The `OscRouter` handles:
- Pattern matching OSC addresses using regex
- Forwarding messages to configured targets
- Address rewriting with capture groups
- Stop propagation for exclusive routing

### Server Configuration (`server::config`)

The `OscServerConfig` provides:
- YAML/TOML configuration file loading
- Auto-target creation from route forwards
- Configuration validation with meaningful error messages
- Default values using derivative crate


## Usage Example

```rust
use ss_osc::server::connection_manager::{OscConnectionManager, OscTarget, OscTransport};
use ss_osc::server::router::{OscRouter, OscRouterRoute, OscRouterRouteForward};
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use regex::Regex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connection manager
    let connection_manager = OscConnectionManager::new();

    // Add OSC targets
    let vrchat_target = OscTarget {
        name: "vrchat".to_string(),
        transport: OscTransportConfig::Udp(OscTransportUdpConfig {
            to: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000),
            from: Some(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8000)), // Bind to specific port
        }),
        reconnect_interval: Duration::from_secs(5),
    };
    connection_manager.add_target(vrchat_target);

    // Connect to all targets
    connection_manager.connect_all().await?;

    // Start connection monitoring (auto-reconnect)
    connection_manager.start_connection_monitor().await?;

    // Create router with forward targets from connection manager
    let forward_targets = connection_manager.get_forward_targets();
    let routes = vec![
        OscRouterRoute {
            address: Regex::new(r"^/avatar/parameters/(?P<param>.*)$")?,
            stop_propagation: false,
            forward: vec![OscRouterRouteForward {
                to: "vrchat".to_string(),
                rewrite: Some("/avatar/parameters/$param".to_string()),
            }],
        },
    ];
    
    let router = OscRouter::new(routes, forward_targets);

    // Route OSC packets
    let from_addr: SocketAddr = "127.0.0.1:8000".parse()?;
    let packet = rosc::OscPacket::Message(rosc::OscMessage {
        addr: "/avatar/parameters/VelocityX".to_string(),
        args: vec![rosc::OscType::Float(0.5)],
    });
    router.route(&packet, &from_addr).await;

    // Graceful shutdown
    connection_manager.shutdown().await;
    
    Ok(())
}
```

## Cancellation Token Architecture

The connection manager uses a hierarchical cancellation token system:

- **Parent Token**: The main connection manager has a root cancellation token
- **Child Tokens**: Each individual connection gets a child token from `parent_token.child_token()`
- **Automatic Propagation**: When the parent token is cancelled (via `shutdown()`), all child tokens are automatically cancelled
- **Individual Control**: Each connection can be cancelled independently by calling `cancel()` on its child token
- **Clean Resource Cleanup**: Tasks monitor their tokens and gracefully shut down when cancelled

This approach eliminates the need for complex MPSC signaling and provides better control over connection lifecycles.

## Persistent Routing Architecture

A critical feature of this connection manager is that **router connections remain valid across reconnections**:

### The Problem
Without careful design, when a connection fails and reconnects:
1. **Initial Connection**: Manager creates `Sender A`, passes to router
2. **Connection Fails**: `Sender A` becomes invalid
3. **Reconnection**: Manager creates `Sender B` internally
4. **Router Still Uses `Sender A`**: Router continues sending to invalid sender ❌

### The Solution: Proxy Senders
The connection manager uses **persistent proxy senders**:

1. **Target Added**: Creates a persistent proxy sender that lives for the target's entire lifetime
2. **Router Gets Proxy**: Router receives the proxy sender that never changes
3. **Connection Fails**: Internal connection sender becomes invalid
4. **Proxy Forwards**: Proxy automatically forwards packets to the current active connection
5. **Reconnection**: New internal sender created, proxy starts forwarding to it seamlessly ✅

### Benefits
- **No Router Updates**: Router never needs to be reconfigured after reconnections
- **Transparent Failover**: Packets continue flowing as soon as connection is restored  
- **Clean Architecture**: Separation of concerns between routing logic and connection management
- **Zero Packet Loss**: Packets are queued in the proxy during brief reconnection periods

## UDP Local Binding

For UDP connections, you can specify a local bind address using the `from` field in `OscTarget`:

- **`from: None`**: Let the OS choose any available port (default behavior)
- **`from: Some(SocketAddr)`**: Bind to a specific local IP and port

### Why specify a local bind address?

1. **Firewall Configuration**: Some applications expect OSC traffic from specific ports
2. **Network Routing**: Control which network interface to use in multi-homed systems  
3. **Port Consistency**: Ensure predictable source ports for debugging/logging
4. **Application Requirements**: Some OSC targets (like VRChat) may have specific expectations

### Example:
```rust
// Bind to specific port - useful for VRChat or other applications expecting traffic from a known port
OscTarget {
    name: "vrchat".to_string(),
    transport: OscTransportConfig::Udp(OscTransportUdpConfig {
        to: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000),
        from: Some(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8000)), // Send from port 8000
    }),
    reconnect_interval: Duration::from_secs(5),
}

// Let OS choose port - simpler configuration  
OscTarget {
    name: "ableton".to_string(),
    transport: OscTransportConfig::Udp(OscTransportUdpConfig {
        to: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9002),
        from: None, // OS chooses available port
    }),
    reconnect_interval: Duration::from_secs(10),
}
```

## Key Components

### OscTarget
Defines a target OSC endpoint with:
- `name`: Unique identifier for the target  
- `transport`: Transport configuration (see OscTransportConfig)
- `reconnect_interval`: How long to wait before reconnect attempts

### OscTransportConfig
Enum defining transport-specific configurations:

#### UDP Transport (`OscTransportConfig::Udp`)
```rust
OscTransportUdpConfig {
    to: SocketAddr,             // Remote target address
    from: Option<SocketAddr>,   // Local bind address (None = OS chooses)
}
```

#### TCP Transport (`OscTransportConfig::Tcp`)
```rust
OscTransportTcpConfig {
    to: SocketAddr,             // Remote target address  
}
```

This design allows easy extension for future transports like WebSocket, with each transport having its own specific configuration fields.

### OscConnectionManager
- `add_target()`: Add a target to be managed
- `connect_all()`: Connect to all configured targets
- `start_connection_monitor()`: Start background task for auto-reconnection
- `get_forward_targets()`: Get MPSC senders for the router
- `shutdown()`: Gracefully shutdown all connections

### OscRouter
- Routes OSC packets based on regex patterns
- Supports address rewriting with capture groups
- Can forward to multiple targets per route
- Supports stop propagation for exclusive routing

## Integration with Existing OSC Server

This connection manager is designed to work with the existing OSC router pipeline. The connection manager provides the MPSC channels that the router expects for forwarding packets to network targets.

## Dependencies

- `tokio`: Async runtime
- `rosc`: OSC encoding/decoding
- `regex`: Pattern matching
- `tokio-util`: Cancellation tokens
- `dashmap`: Thread-safe hashmaps
- `tracing`: Structured logging