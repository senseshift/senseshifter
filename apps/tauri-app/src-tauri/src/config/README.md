# Configuration Module

A reusable configuration module for the Tauri app that provides persistent storage of configuration data using YAML files.

## Features

- **Persistent Storage**: Configurations are stored in the app's data directory
- **Module-based**: Each module can have its own configuration file
- **Default Fallback**: Automatically creates default configurations if files don't exist
- **YAML Format**: Human-readable configuration files
- **Type Safety**: Fully typed configurations using Rust's type system

## Directory Structure

Configurations are stored in the app's data directory under `config/`:

```
~/.senseshifter/config/
├── osc/
│   └── server.yml
├── audio/
│   └── settings.yml
└── ui/
    └── preferences.yml
```

## Usage

### Basic Usage

```rust
use crate::config::ConfigManager;

// Create config manager (typically done in app setup)
let config_manager = ConfigManager::new(&app_handle)?;

// Load configuration for a module
let osc_config: OscServerModuleConfig = config_manager.load_config("osc/server").await?;

// Modify the configuration
let mut modified_config = osc_config;
modified_config.enabled = true;

// Save the modified configuration
config_manager.save_config("osc/server", &modified_config).await?;
```

### Module Path Examples

- `"osc/server"` → `~/.senseshifter/config/osc/server.yml`
- `"audio/settings"` → `~/.senseshifter/config/audio/settings.yml`
- `"ui/preferences"` → `~/.senseshifter/config/ui/preferences.yml`

### Tauri Commands

The module provides Tauri commands for frontend integration:

```javascript
// Load configuration from frontend
const config = await invoke('load_osc_config');

// Save configuration from frontend
await invoke('save_osc_config', { config: modifiedConfig });
```

## Configuration Types

All configuration types must implement:
- `Serialize` + `Deserialize` for YAML serialization
- `Default` for fallback when files don't exist
- `Debug` for logging

Example:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MyModuleConfig {
    pub enabled: bool,
    pub port: u16,
    pub settings: HashMap<String, String>,
}

impl Default for MyModuleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8080,
            settings: HashMap::new(),
        }
    }
}
```

## Error Handling

The module uses `anyhow::Result` for comprehensive error handling:
- File I/O errors
- YAML parsing errors
- Directory creation errors

All errors include context information for debugging.