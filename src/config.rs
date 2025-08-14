use anyhow::Result;
use log::{debug, error, trace, warn};
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
    // Directory to store cached files
    cache_dir: String,
    // Host to route to
    routes: HashMap<String, ProxyRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRoute {
    host: String,
    path: String,
    port: u16,
    protocol: String,
    redirect_to_https: bool,
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
            cache_dir: "./cache".to_string(),
            routes: HashMap::from([(
                "example.com".to_string(),
                ProxyRoute {
                    host: "localhost".to_string(),
                    path: String::new(),
                    port: 8080,
                    protocol: "http".to_string(),
                    redirect_to_https: false,
                },
            )]),
        }
    }

    pub fn get_email(&self) -> &String {
        &self.email
    }
    pub fn get_cache_dir(&self) -> &String {
        &self.cache_dir
    }
    pub fn get_routes(&self) -> &HashMap<String, ProxyRoute> {
        &self.routes
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
    pub fn lookup_host(&self, key: impl AsRef<str>) -> Option<&ProxyRoute> {
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
            let result = serde_json::from_str::<Config>(&content);
            let cfg = if let Err(e) = result {
                error!("Failed to parse config file: {}", e);
                Self::save_default(path).await?;
                Self::new()
            } else if let Ok(cfg) = result {
                cfg
            } else {
                unreachable!()
            };
            cfg
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
impl ProxyRoute {
    pub fn get_host(&self) -> &String {
        &self.host
    }
    pub fn get_path(&self) -> &String {
        &self.path
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
    pub fn get_protocol(&self) -> &String {
        &self.protocol
    }
    pub fn get_redirect_to_https(&self) -> bool {
        self.redirect_to_https
    }
    pub fn get_full_url(&self) -> String {
        format!(
            "{}://{}:{}{}",
            self.protocol, self.host, self.port, self.path
        )
    }
}
