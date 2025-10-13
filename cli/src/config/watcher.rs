use crate::config::types::Config;
use log::{debug, trace, warn};

impl Config {
    /// Start watching the configuration file for changes and reload automatically
    pub fn watch_config_file(&self) {
        use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
        let path = self.path.clone();
        tokio::spawn(async move {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default()).unwrap();
            watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();
            for res in rx {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                        trace!("Config file changed: {:?}", event);
                        debug!("Config file changed, reloading");
                        if let Err(e) = Self::try_load(&path).await {
                            warn!("Failed to reload config: {}", e);
                        }
                    } else {
                        trace!("Config file event: {:?}", event);
                        continue; // ignore other events
                    }
                } else {
                    warn!("Failed to receive config file event: {:?}", res);
                    continue;
                }
            }
        });
    }
}