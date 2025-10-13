use crate::config::types::Config;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

// Global state management with OnceLock
static LOADED_CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
static CONFIG_TX: OnceLock<broadcast::Sender<Config>> = OnceLock::new();

/// Get the global config lock
pub fn config_lock() -> &'static RwLock<Config> {
    LOADED_CONFIG.get_or_init(|| RwLock::new(Config::default()))
}

/// Get the global config broadcaster
pub fn broadcaster() -> &'static broadcast::Sender<Config> {
    CONFIG_TX.get_or_init(|| {
        let (tx, _rx) = broadcast::channel::<Config>(16);
        tx
    })
}

impl Config {
    /// Get a clone of the current global configuration
    pub async fn get() -> Self {
        config_lock().read().await.clone()
    }

    /// Subscribe to configuration changes
    pub fn subscribe() -> broadcast::Receiver<Config> {
        broadcaster().subscribe()
    }
}