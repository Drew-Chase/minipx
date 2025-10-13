use crate::config::types::Config;
use crate::config::manager::{config_lock, broadcaster};
use crate::ipc;
use crate::utils::validation::is_empty_or_whitespace;
use anyhow::Result;
use log::{debug, error, trace, warn};
use std::path::Path;

impl Config {
    /// Resolve the config path from a command line argument or running instance
    pub async fn resolve_config_path(arg: Option<String>) -> String {
        #[allow(clippy::collapsible_if)]
        if let Some(s) = arg {
            if !is_empty_or_whitespace(&s) {
                return s;
            }
        }
        if let Some(path) = ipc::get_running_config_path().await {
            return path;
        }
        "./minipx.json".to_string()
    }

    /// Load configuration from a file, updating global state and broadcasting changes
    pub async fn try_load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        debug!("Loading config from: {}", path.display());
        let config = if path.exists() {
            let content = tokio::fs::read_to_string(path).await?;
            let result = serde_json::from_str::<Config>(&content);
            if let Err(e) = result {
                error!("Failed to parse config file: {}", e);
                // Move the corrupted config file to a backup
                let mut number_of_coruptions = 1;
                let mut backup_path = path.with_extension(format!("corrupted.{}", number_of_coruptions));

                while backup_path.exists() {
                    backup_path = path.with_extension(format!("corrupted.{}", number_of_coruptions));
                    number_of_coruptions += 1;
                }
                std::fs::rename(path, backup_path)?;

                warn!("Config file corrupted, using default config");
                Self::save_default(path).await?;
                Self::new(path)
            } else {
                let mut cfg = result?;
                cfg.path = path.to_owned();
                cfg
            }
        } else {
            warn!("Config file not found, using default config");
            Self::save_default(path).await?;
            Self::new(path)
        };
        trace!("Loaded config: {:#?}", config);

        {
            let mut guard = config_lock().write().await;
            *guard = config.clone();
        }

        let _ = broadcaster().send(config.clone());

        Ok(config)
    }


    /// Save the current configuration to its file
    pub async fn save(&self) -> Result<()> {
        debug!("Saving config to: {}", self.path.display());
        if !self.path.exists() {
            std::fs::create_dir_all(self.path.parent().ok_or(anyhow::anyhow!("Failed to create parent directory for config file"))?)?;
            tokio::fs::File::create(&self.path).await?;
        }
        let content = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&self.path, content).await?;
        Ok(())
    }

    /// Save a default configuration to the specified path
    pub async fn save_default(path: impl AsRef<Path>) -> Result<()> {
        debug!("Saving default config to: {}", path.as_ref().display());
        let path = path.as_ref();
        Self::new(path).save().await?;
        Ok(())
    }
}