//! Integrated OSC Server Example
//! 
//! This example demonstrates the complete integrated OSC system:
//! - OSC server task that listens for incoming packets
//! - Router that processes and forwards packets based on rules
//! - Connection manager that maintains connections to targets
//! - Event emission for external listeners
//! 
//! The flow is:
//! 1. OSC server receives UDP packets
//! 2. Router processes packets and forwards to appropriate targets
//! 3. Connection manager sends packets to configured destinations
//! 4. Events are emitted for any external listeners
//! 
//! Run with: `cargo run --example integrated_server`

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use regex::Regex;
use rosc::{OscMessage, OscPacket, OscType};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{info, debug};

use ss_osc::server::connection_manager::{ConnectionManager, Target};
use ss_osc::server::router::{OscRouterRouteRuntime, OscRouterRouteForwardRuntime};
use ss_osc::server::task::{OscServerTask, OscServerEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Integrated OSC Server Example");
    println!("=================================");

    // Create connection manager with targets
    println!("\n📋 Setting up connection manager...");
    let connection_manager = ConnectionManager::new();
    
    // Add targets that the router will forward to
    let targets = vec![
        Target::udp(
            "vrchat".to_string(),
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000),
            Duration::from_secs(5)
        ),
        Target::tcp(
            "touchdesigner".to_string(),
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9001),
            Duration::from_secs(3)
        ),
        Target::udp(
            "ableton".to_string(),
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9002),
            Duration::from_secs(10)
        ),
    ];

    for target in targets {
        println!("  ➕ Adding target: {} ({})", target.name, target.transport.transport_type());
        let _sender = connection_manager.add_target(target)?;
    }

    // Define routing rules
    println!("\n🎯 Setting up routing rules...");
    let routes = vec![
        // Route 1: VRChat Avatar Parameters
        OscRouterRouteRuntime {
            address: Regex::new(r"^/avatar/parameters/(?P<param>.*)$")?,
            stop_propagation: false,
            forward: vec![OscRouterRouteForwardRuntime {
                to: "vrchat".to_string(),
                rewrite: Some("/avatar/parameters/$param".to_string()),
            }],
        },
        
        // Route 2: Audio Analysis → Ableton + TouchDesigner
        OscRouterRouteRuntime {
            address: Regex::new(r"^/audio/(?P<param>.*)$")?,
            stop_propagation: false,
            forward: vec![
                OscRouterRouteForwardRuntime {
                    to: "ableton".to_string(),
                    rewrite: Some("/live/$param".to_string()),
                },
                OscRouterRouteForwardRuntime {
                    to: "touchdesigner".to_string(),
                    rewrite: Some("/viz/$param".to_string()),
                },
            ],
        },
        
        // Route 3: Everything else → TouchDesigner for monitoring
        OscRouterRouteRuntime {
            address: Regex::new(r"(.*)$")?,
            stop_propagation: false,
            forward: vec![OscRouterRouteForwardRuntime {
                to: "touchdesigner".to_string(),
                rewrite: None,
            }],
        },
    ];

    println!("  🎯 {} routing rules configured", routes.len());

    // Create event channel for server events
    let (event_tx, mut event_rx) = broadcast::channel(100);
    
    // Create cancellation token for graceful shutdown
    let cancellation_token = CancellationToken::new();

    // Create integrated server task
    println!("\n🏗️  Creating integrated OSC server...");
    let mut server_task = OscServerTask::with_integrated_routing(
        connection_manager,
        routes,
        vec![SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8000)], // Listen on port 8000
        vec![], // No TCP listeners for now
        cancellation_token.clone(),
        event_tx,
    );

    println!("  ✅ Server task created");

    // Spawn event listener task
    let event_listener = {
        let cancellation_token = cancellation_token.clone();
        tokio::spawn(async move {
            info!("Event listener started");
            loop {
                tokio::select! {
                    event = event_rx.recv() => {
                        match event {
                            Ok(OscServerEvent::InboundPacket { packet, from }) => {
                                info!("📨 Received OSC event from {}: {:?}", from, packet);
                            }
                            Err(e) => {
                                debug!("Event receive error (normal on shutdown): {}", e);
                                break;
                            }
                        }
                    },
                    _ = cancellation_token.cancelled() => {
                        info!("Event listener shutdown requested");
                        break;
                    }
                }
            }
            info!("Event listener stopped");
        })
    };

    // Spawn the server task
    let server_handle = {
        let cancellation_token = cancellation_token.clone();
        tokio::spawn(async move {
            info!("Starting OSC server task...");
            match server_task.run().await {
                Ok(_) => info!("OSC server task completed successfully"),
                Err(e) => info!("OSC server task error: {}", e),
            }
        })
    };

    println!("\n🌐 Server is now running!");
    println!("  📡 Listening on UDP port 8000");
    println!("  🎯 Routing to 3 targets: vrchat, touchdesigner, ableton");
    println!("  🔄 Auto-reconnection enabled");
    println!();
    println!("💡 Test the server by sending OSC messages to 127.0.0.1:8000");
    println!("   Example addresses to try:");
    println!("   • /avatar/parameters/VelocityX → VRChat");
    println!("   • /audio/rms → Ableton + TouchDesigner");
    println!("   • /test/message → TouchDesigner (catchall)");
    println!();

    // Simulate some test packets after a delay
    println!("⏱️  Sending test packets in 2 seconds...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    println!("🧪 Sending test OSC packets...");
    
    // Create a simple OSC client to send test packets
    let test_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await?;
    let server_addr: SocketAddr = "127.0.0.1:8000".parse()?;
    
    // Test packet 1: Avatar parameter
    let avatar_msg = OscMessage {
        addr: "/avatar/parameters/VelocityX".to_string(),
        args: vec![OscType::Float(0.5)],
    };
    let avatar_packet = OscPacket::Message(avatar_msg);
    let encoded = rosc::encoder::encode(&avatar_packet)?;
    test_socket.send_to(&encoded, server_addr).await?;
    println!("  📤 Sent avatar parameter to server");
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Test packet 2: Audio data
    let audio_msg = OscMessage {
        addr: "/audio/rms".to_string(),
        args: vec![OscType::Float(0.8)],
    };
    let audio_packet = OscPacket::Message(audio_msg);
    let encoded = rosc::encoder::encode(&audio_packet)?;
    test_socket.send_to(&encoded, server_addr).await?;
    println!("  📤 Sent audio data to server");
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Test packet 3: Catchall
    let test_msg = OscMessage {
        addr: "/test/message".to_string(),
        args: vec![OscType::String("Hello World".to_string())],
    };
    let test_packet = OscPacket::Message(test_msg);
    let encoded = rosc::encoder::encode(&test_packet)?;
    test_socket.send_to(&encoded, server_addr).await?;
    println!("  📤 Sent test message to server");

    // Let it run for 5 seconds to process packets and show activity
    println!("\n⏱️  Running for 5 seconds to demonstrate packet processing...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Graceful shutdown
    println!("\n🛑 Initiating graceful shutdown...");
    cancellation_token.cancel();
    
    // Wait for tasks to complete
    let _ = tokio::try_join!(server_handle, event_listener);
    
    println!("✅ Shutdown complete!");
    
    println!("\n🎉 Integrated server example completed!");
    println!("\n📊 What happened:");
    println!("  • OSC server listened on UDP port 8000");
    println!("  • Router processed incoming packets with 3 rules");
    println!("  • Connection manager handled forwarding to targets");
    println!("  • Events were emitted for each received packet");
    println!("  • System shut down gracefully when requested");
    
    println!("\nNOTE: Connection attempts to targets are expected to fail");
    println!("      unless you have OSC servers running on the target ports.");

    Ok(())
}