//! Router Integration Example
//! 
//! This example demonstrates the complete integration of the OSC router 
//! with the connection manager to create a sophisticated OSC routing system:
//! - Route incoming OSC messages based on address patterns
//! - Forward messages to multiple targets with address rewriting
//! - Handle different transport protocols (UDP/TCP) transparently
//! - Demonstrate real-world routing scenarios for creative applications
//! 
//! Run with: `cargo run --example router_integration`

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use regex::Regex;
use rosc::{OscMessage, OscPacket, OscType};

use ss_osc::server::connection_manager::{
    ConnectionManager, Target, TransportConfig, 
    UdpTransportConfig, TcpTransportConfig
};
use ss_osc::server::router::{OscRouter, OscRouterRouteRuntime, OscRouterRouteForwardRuntime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🎛️  OSC Router Integration Example");
    println!("==================================");

    // Step 1: Set up connection manager with targets
    println!("\n📋 Setting up OSC routing targets...");
    
    let connection_manager = ConnectionManager::new();
    
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
        Target {
            name: "reaper".to_string(),
            transport: TransportConfig::Udp(UdpTransportConfig {
                to: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8000),
            }),
            reconnect_interval: Duration::from_secs(5),
        },
    ];

    // Add targets to connection manager
    for target in targets {
        println!("  ➕ Adding target: {} ({})", target.name, target.transport.transport_type());
        let _sender = connection_manager.add_target(target)?;
    }

    // Attempt to connect (will fail without running servers, but that's OK for the demo)
    match connection_manager.connect_all().await {
        Ok(_) => println!("  ✅ All targets connected successfully"),
        Err(e) => println!("  ⚠️  Connection failed (expected): {}", e),
    }

    // Start connection monitoring
    connection_manager.start_connection_monitor().await?;
    
    // Get forward targets for the router
    let forward_targets = connection_manager.get_forward_targets();
    println!("  📡 {} targets available for routing", forward_targets.len());

    // Step 2: Define sophisticated routing rules
    println!("\n🎯 Setting up routing rules...");
    
    let routes = vec![
        // Route 1: VRChat Avatar Parameters
        // /avatar/parameters/* → VRChat (unchanged)
        OscRouterRouteRuntime {
            address: Regex::new(r"^/avatar/parameters/(?P<param>.*)$")?,
            stop_propagation: false,
            forward: vec![OscRouterRouteForwardRuntime {
                to: "vrchat".to_string(),
                rewrite: Some("/avatar/parameters/$param".to_string()),
            }],
        },
        
        // Route 2: Haptic Feedback Distribution
        // /haptic/device/actuator → TouchDesigner for visualization + VRChat for avatar response
        OscRouterRouteRuntime {
            address: Regex::new(r"^/haptic/(?P<device>.*)/(?P<actuator>.*)$")?,
            stop_propagation: false,
            forward: vec![
                // Send to TouchDesigner for real-time visualization
                OscRouterRouteForwardRuntime {
                    to: "touchdesigner".to_string(),
                    rewrite: Some("/viz/haptic/$device/$actuator".to_string()),
                },
                // Send to VRChat for avatar haptic feedback
                OscRouterRouteForwardRuntime {
                    to: "vrchat".to_string(),
                    rewrite: Some("/avatar/parameters/Haptic_$device".to_string()),
                },
            ],
        },
        
        // Route 3: Audio Analysis Data
        // /audio/analysis/* → Ableton (for reactive music) + TouchDesigner (for visuals)
        OscRouterRouteRuntime {
            address: Regex::new(r"^/audio/analysis/(?P<param>.*)$")?,
            stop_propagation: false,
            forward: vec![
                OscRouterRouteForwardRuntime {
                    to: "ableton".to_string(),
                    rewrite: Some("/live/song/view/$param".to_string()),
                },
                OscRouterRouteForwardRuntime {
                    to: "touchdesigner".to_string(),
                    rewrite: Some("/viz/audio/$param".to_string()),
                },
            ],
        },
        
        // Route 4: MIDI-like Control Messages  
        // /control/cc/* → Ableton + Reaper
        OscRouterRouteRuntime {
            address: Regex::new(r"^/control/cc/(?P<channel>\\d+)/(?P<cc>\\d+)$")?,
            stop_propagation: false,
            forward: vec![
                OscRouterRouteForwardRuntime {
                    to: "ableton".to_string(),
                    rewrite: Some("/live/song/view/track/$channel/mixer/volume".to_string()),
                },
                OscRouterRouteForwardRuntime {
                    to: "reaper".to_string(),
                    rewrite: Some("/track/$channel/volume".to_string()),
                },
            ],
        },
        
        // Route 5: Biometric Data Distribution
        // /bio/* → All targets with different transformations
        OscRouterRouteRuntime {
            address: Regex::new(r"^/bio/(?P<sensor>.*)$")?,
            stop_propagation: false,
            forward: vec![
                // Heart rate affects avatar breathing
                OscRouterRouteForwardRuntime {
                    to: "vrchat".to_string(),
                    rewrite: Some("/avatar/parameters/HeartRate".to_string()),
                },
                // Biometric data drives visual effects
                OscRouterRouteForwardRuntime {
                    to: "touchdesigner".to_string(),
                    rewrite: Some("/viz/bio/$sensor".to_string()),
                },
                // Biometric data affects music tempo/mood
                OscRouterRouteForwardRuntime {
                    to: "ableton".to_string(),
                    rewrite: Some("/live/song/tempo_offset".to_string()),
                },
            ],
        },
        
        // Route 6: System Monitoring → TouchDesigner only
        OscRouterRouteRuntime {
            address: Regex::new(r"^/system/(?P<metric>.*)$")?,
            stop_propagation: true, // Stop here, don't continue to catchall
            forward: vec![OscRouterRouteForwardRuntime {
                to: "touchdesigner".to_string(),
                rewrite: Some("/system/$metric".to_string()),
            }],
        },
        
        // Route 7: Catchall Route - send everything else to TouchDesigner for monitoring
        OscRouterRouteRuntime {
            address: Regex::new(r"(.*)$")?,
            stop_propagation: false,
            forward: vec![OscRouterRouteForwardRuntime {
                to: "touchdesigner".to_string(),
                rewrite: None, // Keep original address
            }],
        },
    ];

    println!("  🎯 {} routing rules configured", routes.len());

    // Step 3: Create router with forward targets
    let router = OscRouter::new(routes, forward_targets);
    
    // Step 4: Simulate realistic OSC traffic
    println!("\n🎵 Simulating realistic OSC traffic...");
    
    let from_addr: SocketAddr = "127.0.0.1:8000".parse()?;
    
    // Simulate VRChat avatar parameters
    println!("\n  👤 VRChat Avatar Parameters:");
    let avatar_packets = vec![
        ("/avatar/parameters/VelocityX", OscType::Float(0.5)),
        ("/avatar/parameters/VelocityZ", OscType::Float(-0.3)),
        ("/avatar/parameters/InStation", OscType::Bool(false)),
        ("/avatar/parameters/Gesture", OscType::Int(2)),
    ];
    
    for (addr, arg) in avatar_packets {
        let packet = OscPacket::Message(OscMessage {
            addr: addr.to_string(),
            args: vec![arg],
        });
        println!("    📤 {} → VRChat", addr);
        router.route(&packet, &from_addr).await;
    }
    
    // Simulate haptic feedback data
    println!("\n  🎮 Haptic Feedback:");
    let haptic_packets = vec![
        ("/haptic/vest/motor_1", vec![OscType::Float(0.8), OscType::Int(100)]),
        ("/haptic/gloves/finger_0", vec![OscType::Float(0.6), OscType::Int(200)]),
        ("/haptic/shoes/heel_left", vec![OscType::Float(1.0), OscType::Int(50)]),
    ];
    
    for (addr, args) in haptic_packets {
        let packet = OscPacket::Message(OscMessage {
            addr: addr.to_string(),
            args,
        });
        println!("    📤 {} → TouchDesigner + VRChat", addr);
        router.route(&packet, &from_addr).await;
    }
    
    // Simulate audio analysis data
    println!("\n  🎧 Audio Analysis:");
    let audio_packets = vec![
        ("/audio/analysis/rms", OscType::Float(0.45)),
        ("/audio/analysis/pitch", OscType::Float(440.0)),
        ("/audio/analysis/onset", OscType::Bool(true)),
        ("/audio/analysis/beat", OscType::Int(1)),
    ];
    
    for (addr, arg) in audio_packets {
        let packet = OscPacket::Message(OscMessage {
            addr: addr.to_string(),
            args: vec![arg],
        });
        println!("    📤 {} → Ableton + TouchDesigner", addr);
        router.route(&packet, &from_addr).await;
    }
    
    // Simulate MIDI-like control
    println!("\n  🎹 MIDI Control:");
    let control_packets = vec![
        ("/control/cc/1/7", OscType::Float(0.75)), // Channel 1, CC 7 (volume)
        ("/control/cc/2/1", OscType::Float(0.5)),  // Channel 2, CC 1 (modulation)
        ("/control/cc/3/64", OscType::Float(1.0)), // Channel 3, CC 64 (sustain)
    ];
    
    for (addr, arg) in control_packets {
        let packet = OscPacket::Message(OscMessage {
            addr: addr.to_string(),
            args: vec![arg],
        });
        println!("    📤 {} → Ableton + Reaper", addr);
        router.route(&packet, &from_addr).await;
    }
    
    // Simulate biometric data
    println!("\n  💓 Biometric Data:");
    let bio_packets = vec![
        ("/bio/heart_rate", OscType::Int(72)),
        ("/bio/skin_conductance", OscType::Float(0.3)),
        ("/bio/breathing_rate", OscType::Int(16)),
        ("/bio/temperature", OscType::Float(98.6)),
    ];
    
    for (addr, arg) in bio_packets {
        let packet = OscPacket::Message(OscMessage {
            addr: addr.to_string(),
            args: vec![arg],
        });
        println!("    📤 {} → All targets", addr);
        router.route(&packet, &from_addr).await;
    }
    
    // Simulate system monitoring
    println!("\n  💻 System Monitoring:");
    let system_packets = vec![
        ("/system/cpu_usage", OscType::Float(0.35)),
        ("/system/memory_usage", OscType::Float(0.62)),
        ("/system/network_latency", OscType::Float(12.5)),
    ];
    
    for (addr, arg) in system_packets {
        let packet = OscPacket::Message(OscMessage {
            addr: addr.to_string(),
            args: vec![arg],
        });
        println!("    📤 {} → TouchDesigner only (stop_propagation=true)", addr);
        router.route(&packet, &from_addr).await;
    }
    
    // Simulate unknown/catchall data
    println!("\n  ❓ Unknown/Catchall Routes:");
    let catchall_packets = vec![
        ("/custom/user/data", OscType::String("hello".to_string())),
        ("/experimental/feature", OscType::Float(0.123)),
        ("/debug/info", OscType::String("debug message".to_string())),
    ];
    
    for (addr, arg) in catchall_packets {
        let packet = OscPacket::Message(OscMessage {
            addr: addr.to_string(),
            args: vec![arg],
        });
        println!("    📤 {} → TouchDesigner (catchall)", addr);
        router.route(&packet, &from_addr).await;
    }

    // Wait for packets to be processed
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Graceful shutdown
    println!("\n🛑 Shutting down...");
    connection_manager.shutdown().await;

    println!("\n🎉 Router integration example completed!");
    println!("\n📊 Summary:");
    println!("  • {} OSC routing rules demonstrated", 7);
    println!("  • {} different target applications", 4);
    println!("  • Both UDP and TCP transports used");
    println!("  • Address rewriting and multi-target forwarding shown");
    println!("  • Stop propagation and catchall routing demonstrated");
    
    println!("\nNOTE: Actual packet delivery requires running OSC servers on:");
    println!("  - VRChat OSC:      127.0.0.1:9000 (UDP)");
    println!("  - TouchDesigner:   127.0.0.1:9001 (TCP)");
    println!("  - Ableton Live:    127.0.0.1:9002 (UDP)");
    println!("  - Reaper:          127.0.0.1:8000 (UDP)");

    Ok(())
}