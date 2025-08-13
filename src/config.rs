use anyhow::Result;
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;
use tokio::sync::RwLock;

static LOADED_CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

fn config_lock() -> &'static RwLock<Config> {
    LOADED_CONFIG.get_or_init(|| RwLock::new(Config::new()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    email: String,
    cache_dir: String,
    routes: HashMap<String, u16>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            email: "email@example.com".to_string(),
            cache_dir: String::new(),
            routes: HashMap::from([("example.com".to_string(), 8080u16)]),
        }
    }
    pub fn get_email(&self) -> &String {
        &self.email
    }
    pub fn get_cache_dir(&self) -> &String {
        &self.cache_dir
    }
    pub fn get_routes(&self) -> &HashMap<String, u16> {
        &self.routes
    }
    pub fn lookup_host(&self, key: impl AsRef<str>) -> Option<&u16> {
        let host = key.as_ref();
        if let Some(port) = self.routes.get(host) {
            return Some(port);
        }
        self.routes.iter()
            .find(|(k, _)| k.starts_with("*.") && host.ends_with(&k[1..]))
            .map(|(_, v)| v)
    }

    pub async fn get() -> Self {
        config_lock().read().await.clone()
    }

    pub async fn try_load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        debug!("Loading config from: {}", path.display());
        let config = if path.exists() {
            let content = tokio::fs::read_to_string(path).await?;
            serde_json::from_str(&content)?
        } else {
            warn!("Config file not found, using default config");
            Self::save_default(path).await?;
            Self::new()
        };
        trace!("Loaded config: {:#?}", config);
        *config_lock().write().await = config;
        Ok(Self::get().await)
    }

    pub async fn save_default(path: impl AsRef<Path>) -> Result<()> {
        debug!("Saving default config to: {}", path.as_ref().display());
        let path = path.as_ref();
        tokio::fs::create_dir_all(path.parent().unwrap()).await?;
        let path = path.with_extension("json");
        let content = serde_json::to_string_pretty(&Config::new())?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    pub fn watch_config_file(path: impl AsRef<Path>) {
        use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
        let path = path.as_ref().to_owned();
        tokio::spawn(async move {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default()).unwrap();
            watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();
            for _res in rx {
                if let Err(e) = Self::try_load(&path).await {
                    warn!("Failed to reload config: {}", e);
                }
            }
        });
    }

}
