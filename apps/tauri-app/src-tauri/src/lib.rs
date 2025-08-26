use std::sync::Arc;
use tokio::sync::mpsc;
use tauri::{Manager, State, Emitter};
use uuid::Uuid;

mod config;
mod osc;

use config::ConfigManager;
use osc::{
    OscServerManager, OscRuntimeConfig, OscServerModuleConfig,
    ServerStatusEvent
};

// Application state
pub struct AppState {
    osc_manager: Arc<OscServerManager>,
    pub(crate) config_manager: Arc<ConfigManager>,
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
    // Toggle in the runtime manager
    state.osc_manager.toggle_global_osc(enabled).await
        .map_err(|e| e.to_string())?;
    
    // Update persistent configuration
    let mut config: OscServerModuleConfig = state.config_manager.load_config("osc/server").await
        .unwrap_or_default();
    config.enabled = enabled;
    
    state.config_manager.save_config("osc/server", &config).await
        .map_err(|e| e.to_string())?;
    
    tracing::info!("Global OSC toggled to {} and config saved", enabled);
    Ok(())
}

#[tauri::command]
async fn toggle_osc_server(
    state: State<'_, AppState>,
    server_id: String,
    enabled: bool,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&server_id).map_err(|e| e.to_string())?;
    
    // Toggle in the runtime manager
    state.osc_manager.toggle_server(uuid, enabled).await
        .map_err(|e| e.to_string())?;
    
    // Update persistent configuration
    let mut config: OscServerModuleConfig = state.config_manager.load_config("osc/server").await
        .unwrap_or_default();
    
    // Find the server index in the configuration
    if let Some(server_index) = state.osc_manager.get_server_index_by_uuid(uuid).await {
        if let Some(server_config) = config.servers.get_mut(server_index) {
            server_config.enabled = enabled;
            
            state.config_manager.save_config("osc/server", &config).await
                .map_err(|e| e.to_string())?;
            
            tracing::info!("OSC server {} toggled to {} and config saved", server_id, enabled);
        } else {
            tracing::warn!("Server index {} not found in config array", server_index);
        }
    } else {
        tracing::warn!("Could not find server index for UUID {}", server_id);
    }
    
    Ok(())
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

#[tauri::command]
async fn load_osc_config(
    state: State<'_, AppState>,
) -> Result<OscServerModuleConfig, String> {
    tracing::info!("load_osc_config called");

    let config = state.config_manager.load_config("osc/server").await
        .unwrap_or_default();

    tracing::info!("load_osc_config returning: {:?}", config);

    Ok(config)
}

#[tauri::command]
async fn save_osc_config(
    state: State<'_, AppState>,
    config: OscServerModuleConfig,
) -> Result<(), String> {
    tracing::info!("save_osc_config called with: {:?}", config);
    state.config_manager.save_config("osc/server", &config).await
        .map_err(|e| e.to_string())?;
    
    // Update runtime config and reinitialize OSC manager
    let runtime_config = OscServerManager::generate_runtime_config(config);
    state.osc_manager.initialize(runtime_config).await
        .map_err(|e| e.to_string())?;
    
    tracing::info!("save_osc_config completed successfully");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tracing::info!("Starting Tauri app setup");
            
            let (event_tx, mut event_rx) = mpsc::unbounded_channel();
            
            // Create configuration manager
            tracing::info!("Creating configuration manager");
            let config_manager = Arc::new(
                ConfigManager::new(app.handle())
                    .expect("Failed to create configuration manager")
            );
            
            // Create OSC manager
            tracing::info!("Creating OSC server manager");
            let osc_manager = Arc::new(OscServerManager::new(event_tx));
            
            // Load config from file and initialize OSC manager
            tracing::info!("Loading configuration and initializing OSC server manager");
            let config_manager_clone = config_manager.clone();
            let osc_manager_clone = osc_manager.clone();
            tauri::async_runtime::block_on(async move {
                match config_manager_clone.load_config::<OscServerModuleConfig>("osc/server").await {
                    Ok(module_config) => {
                        let runtime_config = OscServerManager::generate_runtime_config(module_config);
                        if let Err(e) = osc_manager_clone.initialize(runtime_config).await {
                            tracing::error!("Failed to initialize OSC server manager: {}", e);
                        } else {
                            tracing::info!("OSC server manager initialized successfully");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to load OSC config: {}", e);
                        // Fallback to default config
                        let module_config = OscServerModuleConfig::default();
                        let runtime_config = OscServerManager::generate_runtime_config(module_config);
                        if let Err(e) = osc_manager_clone.initialize(runtime_config).await {
                            tracing::error!("Failed to initialize OSC server manager with default config: {}", e);
                        }
                    }
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
            app.manage(AppState { osc_manager, config_manager });
            
            tracing::info!("Tauri app setup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_osc_config,
            toggle_global_osc,
            toggle_osc_server,
            get_server_statuses,
            load_osc_config,
            save_osc_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
