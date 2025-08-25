use std::collections::HashMap;

use anyhow::Result;
use clap::{Arg, Command};
use tokio::sync::mpsc;
use tracing::{error, info};
use ss_osc::server::{config::OscServerConfig, OscServerBuilder};
use ss_osc::server::connection_manager::{ConnectionEvent, TransportConfig};

mod logging;
mod tui;

use tui::{ConnectionStatus, UiEvent, run_tui};

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
        let sample_config = OscServerConfig::default();
        let yaml_content = serde_yaml::to_string(&sample_config)?;
        println!("# Sample OSC Server Configuration");
        println!("{}", yaml_content);
        return Ok(());
    }

    // Load configuration
    let config_path = matches.get_one::<String>("config")
        .ok_or_else(|| anyhow::anyhow!("Config file path is required"))?;
    let config: OscServerConfig = serde_yaml::from_str(&std::fs::read_to_string(config_path)?)?;
    
    info!("Loaded configuration from {}", config_path);

    // Setup channels for UI communication
    let (ui_tx, ui_rx) = mpsc::unbounded_channel::<UiEvent>();
    
    // Setup logging with TUI integration
    logging::setup_logging(ui_tx.clone())?;

    info!("Starting OSC Proxy with TUI interface");
    
    // Build OSC server
    let builder = OscServerBuilder::new(config);
    let server = builder.build()?;
    
    // Create initial target info for TUI (empty for now, will be populated by events)
    let initial_targets = HashMap::new();

    // Start event handler task
    let ui_tx_clone = ui_tx.clone();
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
    let (_server_events_rx, mut conn_events_rx) = server.subscribe_to_events();
    
    let event_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                event = conn_events_rx.recv() => {
                    match event {
                        Ok(conn_event) => {
                            let (status, target, next_attempt_at) = match &conn_event {
                                ConnectionEvent::Connected { target } => {
                                    info!("Target {} connected", target.name);
                                    (ConnectionStatus::Online, target, None)
                                }
                                ConnectionEvent::Disconnected { target } => {
                                    info!("Target {} disconnected", target.name);
                                    (ConnectionStatus::Offline, target, None)
                                }
                                ConnectionEvent::Reconnecting { target } => {
                                    info!("Target {} reconnecting", target.name);
                                    (ConnectionStatus::Reconnecting, target, None)
                                }
                                ConnectionEvent::Failed { target, next_attempt_at } => {
                                    error!("Target {} connection failed", target.name);
                                    (ConnectionStatus::Failed, target, Some(*next_attempt_at))
                                }
                            };
                            
                            // Extract transport and address info from target config
                            let (transport, address) = match &target.transport {
                                TransportConfig::Udp(udp_config) => {
                                    ("UDP".to_string(), udp_config.to.to_string())
                                }
                                TransportConfig::Tcp(tcp_config) => {
                                    ("TCP".to_string(), tcp_config.to.to_string())
                                }
                            };
                            
                            let ui_event = UiEvent::TargetInfo {
                                name: target.name.clone(),
                                address,
                                transport: transport.clone(),
                                status,
                                next_attempt_at,
                            };
                            
                            if let Err(e) = ui_tx_clone.send(ui_event) {
                                error!("Failed to send UI event: {}", e);
                            }
                        }
                        Err(_) => {
                            // Channel closed or lagged
                            break;
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    info!("Event handler shutdown requested");
                    break;
                }
            }
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
        _ = tui_handle => {
            info!("TUI task completed");
        }
        _ = event_handle => {
            info!("Event handler completed");
        }
    }

    // Shutdown server and event handler
    let _ = shutdown_tx.send(());
    server.shutdown().await;
    
    info!("OSC Proxy shutting down");
    Ok(())
}