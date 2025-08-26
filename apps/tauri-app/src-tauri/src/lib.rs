use std::sync::Arc;
use tokio::sync::mpsc;
use tauri::{Manager, State, Emitter};
use uuid::Uuid;

mod osc;

use osc::{
    OscServerManager, OscRuntimeConfig, OscServerModuleConfig,
    ServerStatusEvent, ConnectionStatusEvent
};

// Application state
pub struct AppState {
    osc_manager: Arc<OscServerManager>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_osc_config(state: State<'_, AppState>) -> Result<OscRuntimeConfig, String> {
    tracing::info!("get_osc_config called");
    let config = state.osc_manager.get_config().await;
    tracing::info!("get_osc_config returning: {:?}", config);
    Ok(config)
}

#[tauri::command]
async fn toggle_global_osc(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    state.osc_manager.toggle_global_osc(enabled).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_osc_server(
    state: State<'_, AppState>,
    server_id: String,
    enabled: bool,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&server_id).map_err(|e| e.to_string())?;
    state.osc_manager.toggle_server(uuid, enabled).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_server_statuses(
    state: State<'_, AppState>,
) -> Result<Vec<ServerStatusEvent>, String> {
    tracing::info!("get_server_statuses called");
    let statuses = state.osc_manager.get_all_server_statuses().await;
    tracing::info!("get_server_statuses returning: {:?}", statuses);
    Ok(statuses)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tracing::info!("Starting Tauri app setup");
            
            let (event_tx, mut event_rx) = mpsc::unbounded_channel();
            
            // Create OSC manager
            tracing::info!("Creating OSC server manager");
            let osc_manager = Arc::new(OscServerManager::new(event_tx));
            
            // Generate runtime config from module config and initialize
            tracing::info!("Generating runtime config and initializing OSC server manager");
            let module_config = OscServerModuleConfig::default();
            let runtime_config = OscServerManager::generate_runtime_config(module_config);
            
            let osc_manager_clone = osc_manager.clone();
            tauri::async_runtime::block_on(async move {
                if let Err(e) = osc_manager_clone.initialize(runtime_config).await {
                    tracing::error!("Failed to initialize OSC server manager: {}", e);
                } else {
                    tracing::info!("OSC server manager initialized successfully");
                }
            });
            
            // Setup event forwarding to frontend
            tracing::info!("Setting up event forwarding");
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    tracing::info!("Forwarding event to frontend: {:?}", event);
                    if let Err(e) = app_handle.emit("osc-event", event) {
                        tracing::error!("Failed to emit event to frontend: {}", e);
                    } else {
                        tracing::info!("Event emitted successfully to frontend");
                    }
                }
            });
            
            // Store state
            tracing::info!("Storing app state");
            app.manage(AppState { osc_manager });
            
            tracing::info!("Tauri app setup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_osc_config,
            toggle_global_osc,
            toggle_osc_server,
            get_server_statuses
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
