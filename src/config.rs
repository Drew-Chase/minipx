use anyhow::Result;
use log::{debug, error, trace, warn};
use notify::EventKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

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
    // Enable or disable the SSL/HTTPS server globally
    #[serde(default = "default_ssl_enabled")]
    ssl_enabled: bool,
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
            ssl_enabled: true,
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
    pub fn get_port(&self) -> u16 {
        self.port
    }
    pub fn lookup_host(&self, key: impl AsRef<str>) -> Option<&ProxyRoute> {
        let host = key.as_ref();
        if let Some(route) = self.routes.get(host) {
            return Some(route);
        }
        self.routes.iter().find(|(k, _)| k.starts_with("*.") && host.ends_with(&k[1..])).map(|(_, v)| v)
    }

    pub async fn get() -> Self {
        config_lock().read().await.clone()
    }

    pub fn subscribe() -> broadcast::Receiver<Config> {
        broadcaster().subscribe()
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
impl ProxyRoute {
    pub fn get_protocol(&self) -> &String {
        &self.protocol
    }
    pub fn get_redirect_to_https(&self) -> bool {
        self.redirect_to_https
    }
    pub fn get_full_url(&self) -> String {
        format!("{}://{}:{}{}", self.protocol, self.host, self.port, self.path)
    }
}

fn default_ssl_enabled() -> bool {
    true
}

impl Config {
    pub fn is_ssl_enabled(&self) -> bool {
        self.ssl_enabled
    }

    pub fn is_email_valid(&self) -> bool {
        let email = self.get_email();
        // very simple validation: one '@', no spaces, local and domain parts non-empty, domain contains '.'
        if email.is_empty() || email.contains(' ') {
            return false;
        }
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        let (local, domain) = (parts[0], parts[1]);
        if local.is_empty() || domain.is_empty() {
            return false;
        }
        if !domain.contains('.') {
            return false;
        }
        // ensure domain is valid-ish
        Self::validate_domain(domain)
    }

    pub fn validate_domain(domain: &str) -> bool {
        // Disallow wildcard entries here; we cannot obtain wildcard certs with TLS-ALPN/HTTP-01
        if domain.starts_with("*.") {
            return false;
        }
        if domain.len() > 253 || !domain.contains('.') {
            return false;
        }
        // Only allow a-z, A-Z, 0-9, '-', '.'; labels 1..=63, cannot start/end with '-'
        let mut last_dot = true;
        for ch in domain.chars() {
            last_dot = ch == '.';
            if !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '.') {
                return false;
            }
        }
        if last_dot {
            return false;
        } // cannot end with a dot
        for label in domain.split('.') {
            if label.is_empty() || label.len() > 63 {
                return false;
            }
            if label.starts_with('-') || label.ends_with('-') {
                return false;
            }
        }
        true
    }

    /// Returns (valid_domains, invalid_domains) for ACME based on current routes.
    pub fn get_valid_domains_for_acme(&self) -> (Vec<String>, Vec<String>) {
        use std::collections::BTreeSet;
        let mut valid_set: BTreeSet<String> = BTreeSet::new();
        let mut invalid: Vec<String> = Vec::new();
        for (domain, route) in &self.routes {
            if domain.starts_with("*.") {
                invalid.push(domain.clone());
                continue;
            }
            // Only consider routes that intend to serve HTTPS at the frontend
            if !route.protocol.eq_ignore_ascii_case("https") {
                continue; // neither valid nor invalid; just not used for ACME
            }
            if Self::validate_domain(domain) {
                valid_set.insert(domain.clone());
            } else {
                invalid.push(domain.clone());
            }
        }
        (valid_set.into_iter().collect(), invalid)
    }

    /// True if this config can serve TLS for the specific host.
    pub fn can_serve_tls_for_host(&self, host: &str) -> bool {
        if !self.is_ssl_enabled() || !self.is_email_valid() {
            return false;
        }
        // Route must exist and be configured for HTTPS at the frontend
        if let Some(route) = self.lookup_host(host) {
            if !route.get_protocol().eq_ignore_ascii_case("https") {
                return false;
            }
        } else {
            return false;
        }
        let (valid, _invalid) = self.get_valid_domains_for_acme();
        valid.iter().any(|d| d == host)
    }
}
