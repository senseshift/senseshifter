//! Configuration Example
//! 
//! This example demonstrates how to use the configuration system with
//! builder patterns and derivative-based defaults.
//! 
//! Run with: `cargo run --example configuration`

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use ss_osc::server::connection_manager::{
    ConnectionManagerConfig, Target, TransportConfig
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  OSC Configuration System Example");
    println!("====================================");

    // Example 1: Using default configuration
    println!("\n📋 Default Configuration:");
    let default_config = ConnectionManagerConfig::default();
    println!("  Packet buffer size: {}", default_config.packet_buffer_size);
    println!("  Health check interval: {:?}", default_config.health_check_interval);
    println!("  Max concurrent reconnections: {}", default_config.max_concurrent_reconnections);

    // Example 2: Using builder pattern
    println!("\n🔧 Custom Configuration with Builder:");
    let custom_config = ConnectionManagerConfig::builder()
        .packet_buffer_size(2048)
        .health_check_interval(Duration::from_secs(30))
        .max_concurrent_reconnections(5)
        .build();
    
    println!("  Packet buffer size: {}", custom_config.packet_buffer_size);
    println!("  Health check interval: {:?}", custom_config.health_check_interval);
    println!("  Max concurrent reconnections: {}", custom_config.max_concurrent_reconnections);

    // Example 3: Transport configurations
    println!("\n🌐 Transport Configurations:");
    
    // UDP transport
    let udp_transport = TransportConfig::udp(
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000)
    );
    
    println!("  UDP Transport: {} -> {}", 
             udp_transport.transport_type(), 
             udp_transport.remote_address());

    // TCP transport
    let tcp_transport = TransportConfig::tcp(
        SocketAddr::new(Ipv4Addr::new(192, 168, 1, 100).into(), 8000)
    );
    
    println!("  TCP Transport: {} -> {}", 
             tcp_transport.transport_type(), 
             tcp_transport.remote_address());

    // Example 4: Creating targets with different configurations
    println!("\n🎯 Target Configurations:");
    
    // Quick target creation using convenience methods
    let quick_target = Target::udp(
        "quick_target".to_string(),
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000),
        Duration::from_secs(5)
    );
    
    println!("  Quick Target: {} ({})", 
             quick_target.name, 
             quick_target.transport.transport_type());

    // Target creation using builder pattern
    let builder_target = Target::builder("builder_target")
        .udp(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9001))
        .reconnect_interval(Duration::from_secs(10))
        .build()?;
    
    println!("  Builder Target: {} ({}) - reconnect every {:?}", 
             builder_target.name, 
             builder_target.transport.transport_type(),
             builder_target.reconnect_interval);

    // Example 5: More target creation methods
    println!("\n✅ Multiple Target Creation Methods:");
    
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
        Target::builder("custom")
            .tcp(SocketAddr::new(Ipv4Addr::new(10, 0, 0, 1).into(), 8080))
            .reconnect_interval(Duration::from_secs(15))
            .build()?,
    ];
    
    for target in &targets {
        println!("  📤 Target '{}': {} -> {} (reconnect: {:?})",
                 target.name,
                 target.transport.transport_type(),
                 target.transport.remote_address(),
                 target.reconnect_interval);
    }

    // Example 6: Debug output with derivative
    println!("\n🐛 Debug Output (via Derivative):");
    println!("  Config: {:#?}", custom_config);
    println!("  Target: {:#?}", builder_target);

    println!("\n🎉 Configuration example completed!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  • Default configurations via derivative");
    println!("  • Builder patterns for ergonomic construction");
    println!("  • Type-safe transport configuration");
    println!("  • Automatic Debug/Clone implementations");
    println!("  • Flexible target creation methods");

    Ok(())
}