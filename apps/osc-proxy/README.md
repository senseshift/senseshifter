# OSC Proxy

A terminal-based OSC (Open Sound Control) proxy with real-time monitoring and routing capabilities.

## Features

- **Real-time TUI Interface**: Monitor OSC connections and logs in a clean terminal interface
- **OSC Routing**: Route OSC messages between targets with pattern matching and address rewriting  
- **Connection Management**: Automatic connection handling with exponential backoff reconnection
- **Multiple Transports**: Support for UDP and TCP connections
- **Live Monitoring**: View connection status, packet counts, and logs in real-time

## Installation

```bash
cargo build --release
```

## Usage

### Basic Usage

```bash
# Run with a configuration file
osc-proxy -c config.yaml

# Generate a sample configuration
osc-proxy --generate-config > config.yaml
```

### Keyboard Controls

- **q, Esc, Ctrl+C**: Quit application
- **h, F1**: Toggle help
- **↑/↓**: Scroll logs line by line
- **PgUp/PgDn**: Scroll logs page by page
- **Home/End**: Go to top/bottom of logs

## Configuration

The proxy uses YAML or TOML configuration files to define server settings and routing rules.

### Basic Configuration

```yaml
# Server UDP address to bind to
server:
  udp:
    - "127.0.0.1:9001"
  tcp: []

# Routing rules (optional)
routes: []

# Connection manager settings (optional)
connection_manager: {}
```

### OSC Routing Configuration

```yaml
server:
  udp:
    - "127.0.0.1:9001"
  tcp: []

routes:
  # Forward avatar parameters to VRChat
  - address: "^/avatar/parameters/(?P<param>.*)$"
    stop_propagation: false
    forward:
      - type: "udp"
        to: "127.0.0.1:9000"
        rewrite: "/avatar/parameters/${param}"
      
  # Send to TouchDesigner
  - address: "^/touchdesigner/(?P<path>.*)$"
    stop_propagation: false
    forward:
      - type: "udp"
        to: "127.0.0.1:7000"
        
  # Send to Ableton Live with track mapping
  - address: "^/ableton/(?P<track>\\d+)/(?P<param>.*)$"
    stop_propagation: true
    forward:
      - type: "tcp"
        to: "127.0.0.1:8080"
        rewrite: "/live/track/${track}/${param}"
```

### Multi-Target Routing

```yaml
server:
  udp:
    - "127.0.0.1:9001"

routes:
  # Broadcast heartbeat to multiple destinations
  - address: "^/heartbeat$"
    forward:
      - type: "udp"
        to: "127.0.0.1:9000"  # VRChat
      - type: "udp"
        to: "127.0.0.1:7000"  # TouchDesigner
      - type: "tcp"
        to: "127.0.0.1:8000"  # Custom app
        
  # Route specific data to specific targets
  - address: "^/vrchat/(?P<param>.*)$"
    stop_propagation: true
    forward:
      - type: "udp"
        to: "127.0.0.1:9000"
        rewrite: "/avatar/parameters/${param}"
```

## Configuration Reference

### Server Configuration

- **`server.udp`**: Array of UDP addresses to bind the server to
- **`server.tcp`**: Array of TCP addresses to bind the server to (optional)

### Route Configuration

- **`address`**: Regular expression pattern to match OSC addresses
- **`stop_propagation`**: Stop processing more routes after this one matches (default: false)
- **`forward`**: Array of forward targets

### Forward Target Configuration

#### UDP Forward
```yaml
- type: "udp"
  to: "127.0.0.1:9000"
  rewrite: "/new/address"  # Optional address rewriting
```

#### TCP Forward
```yaml
- type: "tcp"
  to: "127.0.0.1:9001"
  rewrite: "/new/address"  # Optional address rewriting
```

### Address Rewriting

Use capture groups in your regex patterns and reference them in rewrite rules:

```yaml
- address: "^/input/(?P<channel>\\d+)/(?P<param>.*)$"
  forward:
    - type: "udp"
      to: "127.0.0.1:9000"
      rewrite: "/ch/${channel}/${param}"
```

## TUI Interface

The terminal interface is divided into two main sections:

### Left Panel (2/3 width) - Logs
- Real-time log output with timestamps
- Color-coded log levels (ERROR, WARN, INFO, DEBUG, TRACE)
- Scrollable with keyboard navigation
- Shows connection events and routing activity

### Right Panel (1/3 width) - Targets
- Live connection status with status indicators:
  - 🟢 **Online**: Target is connected and ready
  - 🟡 **Reconnecting**: Attempting to reconnect 
  - 🔴 **Offline/Failed**: Target is disconnected
- Transport protocol (UDP/TCP) and target address
- Packet count for active connections

## Examples

### VRChat Avatar Parameter Routing
```yaml
server:
  udp: ["127.0.0.1:9001"]

routes:
  - address: "^/avatar/parameters/(?P<param>.*)$"
    forward:
      - type: "udp"
        to: "127.0.0.1:9000"
```

### Multi-Application Routing Hub
```yaml
server:
  udp: ["127.0.0.1:9001"]

routes:
  # Route to VRChat
  - address: "^/vrchat/(?P<param>.*)$"
    forward:
      - type: "udp"
        to: "127.0.0.1:9000"
        rewrite: "/avatar/parameters/${param}"
  
  # Route to TouchDesigner
  - address: "^/td/(?P<path>.*)$"
    forward:
      - type: "udp"
        to: "127.0.0.1:7000"
        rewrite: "/${path}"
  
  # Route to Ableton Live
  - address: "^/daw/(?P<control>.*)$"
    forward:
      - type: "udp"
        to: "127.0.0.1:8000"
        rewrite: "/live/${control}"
```

## TOML Configuration

You can also use TOML format:

```toml
[server]
udp = ["127.0.0.1:9001"]
tcp = []

[[routes]]
address = "^/test/(?P<param>.*)$"
stop_propagation = false

[[routes.forward]]
type = "udp"
to = "127.0.0.1:9000"
rewrite = "/forward/${param}"
```

## Troubleshooting

### Connection Issues
- Check that target addresses are correct and services are running
- Verify firewall settings allow OSC traffic
- Monitor the logs panel for connection error details

### Routing Issues  
- Test your regex patterns with online regex tools
- Check the logs for routing match/miss information
- Verify target addresses in forward configurations

### Performance
- The proxy handles high-frequency OSC traffic efficiently
- Monitor packet counts in the targets panel
- Use `stop_propagation: true` to avoid unnecessary processing

## Architecture

The OSC proxy is built on the `ss-osc` crate which provides:
- **Connection Manager**: Handles target connections with automatic reconnection
- **Router**: Pattern-based message routing with address rewriting
- **Server**: OSC protocol handling for UDP and TCP transports

For more technical details, see the [ss-osc crate documentation](../../crates/ss-osc/README.md).