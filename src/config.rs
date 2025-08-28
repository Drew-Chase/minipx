use crate::ipc;
use anyhow::Result;
use clap::Args;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

static LOADED_CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
// Global broadcaster for config change events
static CONFIG_TX: OnceLock<broadcast::Sender<Config>> = OnceLock::new();

fn config_lock() -> &'static RwLock<Config> {
    LOADED_CONFIG.get_or_init(|| RwLock::new(Config::default()))
}

fn broadcaster() -> &'static broadcast::Sender<Config> {
    CONFIG_TX.get_or_init(|| {
        let (tx, _rx) = broadcast::channel::<Config>(16);
        tx
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub(crate) path: PathBuf,
    // Email address used for ssl certificate
    #[serde(deserialize_with = "string_or_default", default = "String::new")]
    email: String,
    // Directory to store cached files
    #[serde(deserialize_with = "string_or_default", default = "default_cache_dir")]
    cache_dir: String,
    // Host to route to
    #[serde(default)]
    routes: HashMap<String, ProxyRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Args)]
pub struct ProxyRoute {
    #[serde(deserialize_with = "string_or_default", default = "default_host")]
    #[arg(short = 'j', long = "host", default_value = "localhost", help = "The redirect host")]
    host: String,

    #[serde(deserialize_with = "string_or_default", default = "default_path")]
    #[arg(short = 'p', long = "path", default_value = "", help = "Path to route to (e.g. /api/v1)")]
    path: String,

    #[serde(deserialize_with = "u16_or_default", default = "default_port")]
    #[arg(short = 'P', long = "port", help = "Port to route to, cannot be 80 or 443, and must be between 1 and 65535")]
    port: u16,

    #[serde(deserialize_with = "bool_or_default", default)]
    #[arg(short = 's', long = "ssl", default_value = "false", help = "Enable SSL")]
    ssl_enable: bool,

    #[arg(
        short = 'l',
        long = "listen-port",
        help = "Port to listen on for incoming requests, defaults to 80 for HTTP and 443 for HTTPS"
    )]
    #[serde(deserialize_with = "u16_option_or_default", default, skip_serializing_if = "Option::is_none")]
    listen_port: Option<u16>,

    #[serde(deserialize_with = "bool_or_default", default)]
    #[arg(short = 'r', long = "redirect", default_value = "false", help = "Redirect HTTP to HTTPS")]
    redirect_to_https: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new("./minipx.json")
    }
}

impl Config {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let path = path.with_extension("json");

        Self { path, email: String::new(), cache_dir: "./cache".to_string(), routes: HashMap::new() }
    }
    pub fn set_email(&mut self, email: String) {
        self.email = email;
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

    pub async fn resolve_config_path(arg: Option<String>) -> String {
        #[allow(clippy::collapsible_if)]
        if let Some(s) = arg {
            if !s.trim().is_empty() {
                return s;
            }
        }
        if let Some(path) = ipc::get_running_config_path().await {
            return path;
        }
        "./minipx.json".to_string()
    }

    pub async fn try_load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        debug!("Loading config from: {}", path.display());
        let config = if path.exists() {
            let content = tokio::fs::read_to_string(path).await?;
            let result = serde_json::from_str::<Config>(&content);
            if let Err(e) = result {
                error!("Failed to parse config file: {}", e);
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

    pub async fn add_route(&mut self, domain: String, route: impl Into<ProxyRoute>) -> Result<()> {
        let mut route = route.into();
        info!("Adding route: {} -> {}:{}{}", domain, route.host, route.port, route.path);
        if self.routes.contains_key(&domain) {
            return Err(anyhow::anyhow!("Route already exists: {}", domain));
        }
        if route.port == 0 {
            return Err(anyhow::anyhow!("Port must be specified"));
        }
        if route.path.ends_with('/') {
            route.path = route.path.trim_end_matches('/').to_string();
            warn!("Path should not end with '/', will be stripped: {}", route.path);
        }
        self.routes.insert(domain, route);
        Ok(())
    }

    pub async fn remove_route(&mut self, host: impl AsRef<str>) -> Result<()> {
        info!("Removing route: {}", host.as_ref());
        if self.routes.remove(host.as_ref()).is_none() {
            warn!("Route not found: {}", host.as_ref());
        }
        Ok(())
    }

    // Apply a partial update to an existing route identified by domain (the map key).
    pub async fn update_route(&mut self, domain: &str, patch: RoutePatch) -> Result<()> {
        let route =
            self.routes.get_mut(domain).ok_or_else(|| anyhow::anyhow!(format!("Route not found: {}", domain)))?;

        if let Some(host) = patch.host {
            route.host = host;
        }
        if let Some(path) = patch.path {
            route.path = if path.ends_with('/') {
                let path = path.trim_end_matches('/').to_string();
                warn!("Path should not end with '/', will be stripped: {}", path);
                path
            } else {
                path
            };
        }
        if let Some(port) = patch.port {
            if port == 0 {
                return Err(anyhow::anyhow!("Port must be between 1 and 65535"));
            }
            route.port = port;
        }
        if let Some(ssl) = patch.ssl_enable {
            route.ssl_enable = ssl;
        }
        if let Some(redir) = patch.redirect_to_https {
            route.redirect_to_https = redir;
        }
        if let Some(lp) = patch.listen_port {
            // Treat 0 as "unset"
            if lp == 0 {
                route.listen_port = None;
            } else {
                route.listen_port = Some(lp);
            }
        }
        Ok(())
    }

    pub async fn save(&self) -> Result<()> {
        debug!("Saving config to: {}", self.path.display());
        if !self.path.exists() {
            std::fs::create_dir_all(
                self.path.parent().ok_or(anyhow::anyhow!("Failed to create parent directory for config file"))?,
            )?;
            tokio::fs::File::create(&self.path).await?;
        }
        let content = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&self.path, content).await?;
        Ok(())
    }

    pub async fn save_default(path: impl AsRef<Path>) -> Result<()> {
        debug!("Saving default config to: {}", path.as_ref().display());
        let path = path.as_ref();
        Self::new(path).save().await?;
        Ok(())
    }

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutePatch {
    pub host: Option<String>,
    pub path: Option<String>,
    pub port: Option<u16>,
    pub ssl_enable: Option<bool>,
    pub redirect_to_https: Option<bool>,
    pub listen_port: Option<u16>,
}

impl ProxyRoute {
    pub fn is_ssl_enabled(&self) -> bool {
        self.ssl_enable
    }
    pub fn get_redirect_to_https(&self) -> bool {
        self.redirect_to_https
    }

    pub fn get_listen_port(&self) -> Option<u16> {
        self.listen_port
    }

    pub fn get_full_url(&self) -> String {
        format!("http://{}:{}{}", self.host, self.port, self.path)
    }

    // New getters for host, port, path to avoid accessing private fields from other modules
    pub fn get_host(&self) -> &str { &self.host }
    pub fn get_port(&self) -> u16 { self.port }
    pub fn get_path(&self) -> &str { &self.path }
}

impl Config {
    pub fn is_ssl_enabled(&self) -> bool {
        for kv in &self.routes {
            let route = kv.1;
            if route.is_ssl_enabled() {
                return true;
            }
        }
        true
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
        // Disallow wildcard entries here; we cannot get wildcard certs with TLS-ALPN/HTTP-01
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
            if !route.is_ssl_enabled() {
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
            if !route.is_ssl_enabled() {
                return false;
            }
        } else {
            return false;
        }
        let (valid, _invalid) = self.get_valid_domains_for_acme();
        valid.iter().any(|d| d == host)
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string_pretty(self).unwrap();
        writeln!(f, "{}", json)
    }
}

fn string_or_default<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(s) => Ok(s),
        Err(e) => {
            warn!("Failed to deserialize string value: {}, using default", e);
            Ok(String::default())
        }
    }
}

fn default_cache_dir() -> String {
    "./cache".to_string()
}

// Forgiving bool: non-bool types fall back to false.
fn bool_or_default<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match bool::deserialize(deserializer) {
        Ok(b) => Ok(b),
        Err(e) => {
            warn!("Failed to deserialize bool value: {}, using false", e);
            Ok(false)
        }
    }
}

// Forgiving u16: non-integer or out-of-range types fall back to default (typically 0 here).
fn u16_or_default<'de, D>(deserializer: D) -> std::result::Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    match u16::deserialize(deserializer) {
        Ok(n) => Ok(n),
        Err(e) => {
            warn!("Failed to deserialize u16 value: {}, using default", e);
            Ok(u16::default())
        }
    }
}

fn u16_option_or_default<'de, D>(deserializer: D) -> std::result::Result<Option<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<u16>::deserialize(deserializer) {
        Ok(Some(n)) if n > u16::MIN && n < u16::MAX => Ok(Some(n)),
        Ok(_) => {
            warn!("Invalid u16 value, using default None");
            Ok(None)
        }
        Err(e) => {
            warn!("Failed to deserialize u16 option value: {}, using default None", e);
            Ok(None)
        }
    }
}

// Defaults for ProxyRoute fields
fn default_host() -> String {
    "localhost".to_string()
}
fn default_path() -> String {
    "".to_string()
}
fn default_port() -> u16 {
    0
}
