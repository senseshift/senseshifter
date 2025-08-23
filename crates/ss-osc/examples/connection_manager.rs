//! Connection Manager Example
//! 
//! This example demonstrates how to use the OSC connection manager to:
//! - Create and manage multiple OSC targets (UDP and TCP)
//! - Connect to all targets with auto-reconnection
//! - Send OSC packets through persistent proxy senders
//! - Add and remove targets dynamically
//! 
//! Run with: `cargo run --example connection_manager`

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use rosc::{OscMessage, OscPacket, OscType};
use ss_osc::server::connection_manager::{
    ConnectionManager, Target, TransportConfig, 
    UdpTransportConfig, TcpTransportConfig
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🎵 OSC Connection Manager Example");
    println!("==================================");

    // Create connection manager
    let connection_manager = ConnectionManager::new();

    // Define OSC targets for common creative applications
    println!("\n📋 Setting up OSC targets...");
    
    let targets = vec![
        Target {
            name: "vrchat".to_string(),
            transport: TransportConfig::Udp(UdpTransportConfig {
                to: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000),
            }),
            reconnect_interval: Duration::from_secs(5),
        },
        Target {
            name: "touchdesigner".to_string(),
            transport: TransportConfig::Tcp(TcpTransportConfig {
                to: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9001),
            }),
            reconnect_interval: Duration::from_secs(3),
        },
        Target {
            name: "ableton".to_string(),
            transport: TransportConfig::Udp(UdpTransportConfig {
                to: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9002),
            }),
            reconnect_interval: Duration::from_secs(10),
        },
    ];

    // Add targets to connection manager and collect senders
    let mut senders = Vec::new();
    for target in targets {
        println!("  ➕ Adding target: {} ({})", target.name, target.transport.transport_type());
        let sender = connection_manager.add_target(target)?;
        senders.push(sender);
    }

    println!("\n🔗 Connecting to all targets...");
    // Note: In a real application, these connections would fail if the servers aren't running
    // For demo purposes, we'll catch and continue on connection errors
    match connection_manager.connect_all().await {
        Ok(_) => println!("  ✅ All targets connected successfully"),
        Err(e) => println!("  ⚠️  Connection failed (expected): {}", e),
    }

    // Start connection monitoring for auto-reconnection
    println!("\n🔄 Starting connection monitor...");
    connection_manager.start_connection_monitor().await?;
    println!("  ✅ Connection monitor started");

    // Get forward targets (proxy senders that persist across reconnections)
    let forward_targets = connection_manager.get_forward_targets();
    println!("\n📡 Forward targets available: {}", forward_targets.len());
    for (name, _sender) in &forward_targets {
        println!("  📤 {}", name);
    }

    // Demonstrate sending packets through the connection manager
    println!("\n🎵 Sending example OSC packets...");
    
    // Example 1: Avatar parameter for VRChat
    if let Some(vrchat_sender) = forward_targets.get("vrchat") {
        let avatar_packet = OscPacket::Message(OscMessage {
            addr: "/avatar/parameters/VelocityX".to_string(),
            args: vec![OscType::Float(0.5)],
        });
        
        match vrchat_sender.send(avatar_packet).await {
            Ok(_) => println!("  ✅ Sent avatar parameter to VRChat"),
            Err(_) => println!("  ⚠️  Failed to send to VRChat (connection not established)"),
        }
    }

    // Example 2: Control data for TouchDesigner
    if let Some(td_sender) = forward_targets.get("touchdesigner") {
        let control_packet = OscPacket::Message(OscMessage {
            addr: "/viz/brightness".to_string(),
            args: vec![OscType::Float(0.8)],
        });
        
        match td_sender.send(control_packet).await {
            Ok(_) => println!("  ✅ Sent control data to TouchDesigner"),
            Err(_) => println!("  ⚠️  Failed to send to TouchDesigner (connection not established)"),
        }
    }

    // Example 3: Audio parameter for Ableton Live
    if let Some(ableton_sender) = forward_targets.get("ableton") {
        let audio_packet = OscPacket::Message(OscMessage {
            addr: "/live/track/1/volume".to_string(),
            args: vec![OscType::Float(0.75)],
        });
        
        match ableton_sender.send(audio_packet).await {
            Ok(_) => println!("  ✅ Sent audio parameter to Ableton Live"),
            Err(_) => println!("  ⚠️  Failed to send to Ableton Live (connection not established)"),
        }
    }

    // Demonstrate dynamic target management
    println!("\n🔧 Demonstrating dynamic target management...");
    
    // Add a new target dynamically
    let dynamic_target = Target {
        name: "reaper".to_string(),
        transport: TransportConfig::Udp(UdpTransportConfig {
            to: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8000),
        }),
        reconnect_interval: Duration::from_secs(5),
    };
    
    println!("  ➕ Adding dynamic target: reaper");
    let _reaper_sender = connection_manager.add_target(dynamic_target)?;
    
    // Check that it was added
    let updated_targets = connection_manager.get_forward_targets();
    println!("  📡 Total targets now: {}", updated_targets.len());

    // Wait a moment
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Remove a target
    println!("  ➖ Removing target: ableton");
    if let Some(removed) = connection_manager.remove_target("ableton") {
        println!("  ✅ Removed target: {}", removed.name);
    }

    let final_targets = connection_manager.get_forward_targets();
    println!("  📡 Final target count: {}", final_targets.len());

    // Keep the connection manager alive for a bit to show monitoring
    println!("\n⏱️  Running for 5 seconds to demonstrate connection monitoring...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Graceful shutdown
    println!("\n🛑 Shutting down connection manager...");
    connection_manager.shutdown().await;
    println!("  ✅ Connection manager shut down gracefully");

    println!("\n🎉 Example completed successfully!");
    println!("\nNOTE: Connection errors are expected unless you have OSC servers running on:");
    println!("  - VRChat OSC:      127.0.0.1:9000 (UDP)");
    println!("  - TouchDesigner:   127.0.0.1:9001 (TCP)");
    println!("  - Ableton Live:    127.0.0.1:9002 (UDP)");
    println!("  - Reaper:          127.0.0.1:8000 (UDP)");

    Ok(())
}