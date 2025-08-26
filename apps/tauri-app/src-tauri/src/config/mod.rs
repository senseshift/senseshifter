use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tracing::info;

pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {

        // Use C:/Users/<user>/.senseshift/config on Windows
        #[cfg(windows)]
        let config_dir = {
            use std::env;

            let user_profile = env::var("USERPROFILE")
                .context("Failed to get USERPROFILE environment variable")?;
            
            let senseshift_dir = PathBuf::from(user_profile).join(".senseshift");
            
            senseshift_dir.join("config")
        };

        #[cfg(not(windows))]
        let config_dir = {
            let app_data_dir = app_handle
                .path()
                .app_config_dir()
                .context("Failed to get app config directory")?;

            info!("app_config_dir (Unix): {:?}", app_data_dir);

            app_data_dir.join("config")
        };

        info!("app_config_dir: {:?}", config_dir);

        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        Ok(Self { config_dir })
    }

    /// Load configuration for a specific module from a file path
    /// 
    /// # Arguments
    /// * `module_path` - The relative path from config directory (e.g., "osc/server")
    pub async fn load_config<T>(&self, module_path: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Serialize + Default + Debug,
    {
        let config_path = self.get_config_path(module_path);
        
        if !config_path.exists() {
            // Create default config if it doesn't exist
            let default_config = T::default();
            self.save_config(module_path, &default_config).await?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: T = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML config: {}", config_path.display()))?;

        Ok(config)
    }

    /// Save configuration for a specific module to a file path
    /// 
    /// # Arguments
    /// * `module_path` - The relative path from config directory (e.g., "osc/server")
    /// * `config` - The configuration to save
    pub async fn save_config<T>(&self, module_path: &str, config: &T) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let config_path = self.get_config_path(module_path);
        
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let yaml_content = serde_yaml::to_string(config)
            .context("Failed to serialize config to YAML")?;

        fs::write(&config_path, yaml_content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// Get the full path to a configuration file
    /// 
    /// # Arguments
    /// * `module_path` - The relative path from config directory (e.g., "osc/server")
    /// 
    /// # Returns
    /// Full path like `~/.senseshifter/config/osc/server.yml`
    pub fn get_config_path(&self, module_path: &str) -> PathBuf {
        self.config_dir
            .join(module_path)
            .with_extension("yml")
    }

    /// Check if a config file exists for the given module path
    pub fn config_exists(&self, module_path: &str) -> bool {
        self.get_config_path(module_path).exists()
    }

    /// List all available configuration files in the config directory
    pub fn list_configs(&self) -> Result<Vec<String>> {
        let config_dir = &self.config_dir;
        if !config_dir.exists() {
            return Ok(vec![]);
        }

        let mut configs = Vec::new();
        self.collect_configs(&config_dir, &config_dir, &mut configs)?;
        Ok(configs)
    }

    fn collect_configs(&self, base_dir: &Path, current_dir: &Path, configs: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.collect_configs(base_dir, &path, configs)?;
            } else if let Some(extension) = path.extension() {
                if extension == "yml" || extension == "yaml" {
                    let relative_path = path.strip_prefix(base_dir)?.with_extension("");
                    configs.push(relative_path.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        enabled: bool,
        port: u16,
    }

    impl Default for TestConfig {
        fn default() -> Self {
            Self {
                name: "test".to_string(),
                enabled: false,
                port: 8080,
            }
        }
    }

    fn create_test_config_manager() -> (ConfigManager, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_manager = ConfigManager {
            config_dir: temp_dir.path().to_path_buf(),
        };
        (config_manager, temp_dir)
    }

    #[tokio::test]
    async fn test_load_default_config() {
        let (config_manager, _temp_dir) = create_test_config_manager();
        
        let config: TestConfig = config_manager.load_config("test/module").await.unwrap();
        assert_eq!(config, TestConfig::default());
    }

    #[tokio::test]
    async fn test_save_and_load_config() {
        let (config_manager, _temp_dir) = create_test_config_manager();
        
        let test_config = TestConfig {
            name: "custom".to_string(),
            enabled: true,
            port: 9090,
        };

        config_manager.save_config("test/module", &test_config).await.unwrap();
        let loaded_config: TestConfig = config_manager.load_config("test/module").await.unwrap();
        
        assert_eq!(loaded_config, test_config);
    }

    #[test]
    fn test_get_config_path() {
        let (config_manager, temp_dir) = create_test_config_manager();
        
        let path = config_manager.get_config_path("osc/server");
        let expected_path = temp_dir.path().join("config").join("osc").join("server.yml");
        
        assert_eq!(path, expected_path);
    }
}