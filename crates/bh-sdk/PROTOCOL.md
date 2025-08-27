# bH Protocol Observations

## Tools used
- [`examples/ws-logging-server`](./examples/ws-logging-server) - A simple WebSocket server that logs incoming messages to the console.

## Observations Log

### 2025-08-27

1. I've found this GitHub repository under Apache-2.0 license: https://github.com/cercata/pysim2bhap/blob/main/sim2bhap/haptic_player.py.
   Here I can see it uses `ws://localhost:15881/v2/feedbacks` URL to send haptic feedbacks. I made a simple WebSocket server to log incoming messages.