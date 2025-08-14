use anyhow::Result;
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

static LOADED_CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
// Global broadcaster for config change events
static CONFIG_TX: OnceLock<broadcast::Sender<Config>> = OnceLock::new();

fn config_lock() -> &'static RwLock<Config> {
    LOADED_CONFIG.get_or_init(|| RwLock::new(Config::new()))
}

fn broadcaster() -> &'static broadcast::Sender<Config> {
    CONFIG_TX.get_or_init(|| {
        let (tx, _rx) = broadcast::channel::<Config>(16);
        tx
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Email address used for ssl certificate
    email: String,
    // Port to listen on
    port: u16,
    // Host to route to
    routes: HashMap<String, String>,
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
            port: 80,
            routes: HashMap::from([("example.com".to_string(), "http://localhost:8080".to_string())]),
        }
    }

    pub fn get_email(&self) -> &String {
        &self.email
    }
    pub fn get_routes(&self) -> &HashMap<String, String> {
        &self.routes
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
    pub fn lookup_host(&self, key: impl AsRef<str>) -> Option<&String> {
        let host = key.as_ref();
        if let Some(route) = self.routes.get(host) {
            return Some(route);
        }
        self.routes
            .iter()
            .find(|(k, _)| k.starts_with("*.") && host.ends_with(&k[1..]))
            .map(|(_, v)| v)
    }

    pub async fn get() -> Self {
        config_lock().read().await.clone()
    }

    pub fn subscribe() -> broadcast::Receiver<Config> {
        broadcaster().subscribe()
    }

    pub async fn subscribe_with_current() -> (Self, broadcast::Receiver<Config>) {
        (Self::get().await, Self::subscribe())
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

        {
            let mut guard = config_lock().write().await;
            *guard = config.clone();
        }

        let _ = broadcaster().send(config.clone());

        Ok(config)
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
                debug!("Config file changed, reloading");
                if let Err(e) = Self::try_load(&path).await {
                    warn!("Failed to reload config: {}", e);
                }
            }
        });
    }
}
