use std::collections::HashMap;

use anyhow::Result;
use clap::{Arg, Command};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

mod config;
mod logging;
mod tui;

use config::ProxyConfig;
use tui::{ConnectionStatus, TargetInfo, UiEvent, run_tui};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("osc-proxy")
        .version("0.1.0")
        .about("OSC Proxy with TUI monitoring")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path (YAML or TOML)")
                .required(false),
        )
        .arg(
            Arg::new("generate-config")
                .long("generate-config")
                .help("Generate a sample configuration file")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Generate sample config if requested
    if matches.get_flag("generate-config") {
        let sample_config = ProxyConfig::create_sample();
        let yaml_content = serde_yaml::to_string(&sample_config)?;
        println!("# Sample OSC Proxy Configuration");
        println!("{}", yaml_content);
        return Ok(());
    }

    // Load configuration
    let config_path = matches.get_one::<String>("config")
        .ok_or_else(|| anyhow::anyhow!("Config file path is required"))?;
    let config = ProxyConfig::load(config_path)?;
    
    // Validate configuration
    config.validate()?;
    info!("Loaded configuration from {}", config_path);

    // Setup channels for UI communication
    let (ui_tx, ui_rx) = mpsc::unbounded_channel::<UiEvent>();
    
    // Setup logging with TUI integration
    logging::setup_logging(
        &config.logging.level,
        config.logging.file,
        &config.logging.file_path,
        ui_tx.clone(),
    )?;

    info!("Starting OSC Proxy with TUI interface");

    // Convert config to ss-osc types
    let (targets, routes) = config.to_ss_osc_config()?;
    
    // Create initial target info for TUI
    let mut initial_targets = HashMap::new();
    for target in &targets {
        let transport_str = match &target.transport {
            ss_osc::server::connection_manager::TransportConfig::Udp(_udp_config) => {
                format!("UDP")
            }
            ss_osc::server::connection_manager::TransportConfig::Tcp(_tcp_config) => {
                format!("TCP")
            }
        };
        
        let address = match &target.transport {
            ss_osc::server::connection_manager::TransportConfig::Udp(udp_config) => {
                udp_config.to.to_string()
            }
            ss_osc::server::connection_manager::TransportConfig::Tcp(tcp_config) => {
                tcp_config.to.to_string()
            }
        };

        // Find description from original config
        let description = config.targets.get(&target.name)
            .and_then(|t| t.description.clone());

        initial_targets.insert(target.name.clone(), TargetInfo {
            name: target.name.clone(),
            address,
            transport: transport_str,
            status: ConnectionStatus::Offline,
            description,
            last_packet_time: None,
            packet_count: 0,
        });
    }

    // Create cancellation token for graceful shutdown
    let cancellation_token = CancellationToken::new();
    let server_cancel_token = cancellation_token.clone();
    
    // Create event channel for server events  
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(100);
    
    // Create connection manager
    let connection_manager = ss_osc::server::connection_manager::ConnectionManager::new();
    
    // Subscribe to connection events
    let mut connection_event_rx = connection_manager.subscribe_to_events();
    
    // Add targets to connection manager
    for target in targets {
        if let Err(e) = connection_manager.add_target(target) {
            error!("Failed to add target: {}", e);
        }
    }
    
    // Start connection event handler task
    let ui_tx_clone = ui_tx.clone();
    let connection_event_handle = tokio::spawn(async move {
        while let Ok(event) = connection_event_rx.recv().await {
            let (status, target_name) = match &event {
                ss_osc::server::connection_manager::ConnectionEvent::Connected { target_name } => {
                    info!("Target {} connected", target_name);
                    (ConnectionStatus::Online, target_name.clone())
                }
                ss_osc::server::connection_manager::ConnectionEvent::Disconnected { target_name } => {
                    info!("Target {} disconnected", target_name);
                    (ConnectionStatus::Offline, target_name.clone())
                }
                ss_osc::server::connection_manager::ConnectionEvent::Reconnecting { target_name } => {
                    info!("Target {} reconnecting", target_name);
                    (ConnectionStatus::Reconnecting, target_name.clone())
                }
                ss_osc::server::connection_manager::ConnectionEvent::Failed { target_name } => {
                    warn!("Target {} connection failed", target_name);
                    (ConnectionStatus::Failed, target_name.clone())
                }
            };
            
            let ui_event = UiEvent::TargetStatusUpdate {
                name: target_name,
                status,
            };
            
            if let Err(e) = ui_tx_clone.send(ui_event) {
                error!("Failed to send UI event: {}", e);
            }
        }
    });
    
    // Start OSC server task
    let server_handle = tokio::spawn(async move {
        let server_addresses = config.server.udp.clone();
        
        // Create server with integrated routing
        let mut server = ss_osc::server::task::OscServerTask::with_integrated_routing(
            connection_manager,
            routes,
            server_addresses,
            vec![], // no TCP addresses for now
            server_cancel_token.clone(),
            event_tx,
        );
        
        info!("OSC server created successfully");
        if let Err(e) = server.run().await {
            error!("OSC server error: {}", e);
        }
    });

    // Start TUI
    let tui_handle = tokio::spawn(async move {
        if let Err(e) = run_tui(initial_targets, ui_rx).await {
            error!("TUI error: {}", e);
        }
    });

    // Wait for any task to complete or be cancelled
    tokio::select! {
        _ = server_handle => {
            info!("OSC server task completed");
        }
        _ = tui_handle => {
            info!("TUI task completed");
        }
        _ = connection_event_handle => {
            info!("Connection event handler completed");
        }
    }

    // Cancel all tasks
    cancellation_token.cancel();
    
    info!("OSC Proxy shutting down");
    Ok(())
}