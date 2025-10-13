use crate::utils::path::trim_trailing_slash;
use crate::utils::validation::validate_custom_port;
use anyhow::Result;
use log::warn;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub(crate) path: PathBuf,
    // Email address used for ssl certificate
    #[serde(deserialize_with = "string_or_default", default = "String::new")]
    pub(crate) email: String,
    // Directory to store cached files
    #[serde(deserialize_with = "string_or_default", default = "default_cache_dir")]
    pub(crate) cache_dir: String,
    // Host to route to
    #[serde(default)]
    pub(crate) routes: HashMap<String, ProxyRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRoute {
    #[serde(deserialize_with = "string_or_default", default = "default_host")]
    pub(crate) host: String,

    #[serde(deserialize_with = "string_or_default", default = "default_path")]
    pub(crate) path: String,

    #[serde(deserialize_with = "u16_or_default", default = "default_port")]
    pub(crate) port: u16,

    #[serde(deserialize_with = "bool_or_default", default)]
    pub(crate) ssl_enable: bool,

    #[serde(deserialize_with = "u16_option_or_default", default, skip_serializing_if = "Option::is_none")]
    pub(crate) listen_port: Option<u16>,

    #[serde(deserialize_with = "bool_or_default", default)]
    pub(crate) redirect_to_https: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) subroutes: Vec<ProxyPathRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPathRoute {
    #[serde(deserialize_with = "string_or_default", default = "default_path")]
    pub path: String,

    #[serde(deserialize_with = "u16_or_default", default = "default_port")]
    pub port: u16,
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

    pub fn get_path(&self) -> &PathBuf {
        &self.path
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

    pub async fn add_route(&mut self, domain: String, route: impl Into<ProxyRoute>) -> Result<()> {
        use log::{info, warn};

        let mut route = route.into();
        info!("Adding route: {} -> {}:{}{}", domain, route.host, route.port, route.path);
        if self.routes.contains_key(&domain) {
            return Err(anyhow::anyhow!("Route already exists: {}", domain));
        }
        if let Err(err) = validate_custom_port(route.port) {
            return Err(anyhow::anyhow!(err));
        }
        if route.path.ends_with('/') {
            route.path = trim_trailing_slash(route.path);
            warn!("Path should not end with '/', will be stripped: {}", route.path);
        }
        self.routes.insert(domain, route);
        Ok(())
    }

    pub async fn remove_route(&mut self, host: impl AsRef<str>) -> Result<()> {
        use log::{info, warn};

        info!("Removing route: {}", host.as_ref());
        if self.routes.remove(host.as_ref()).is_none() {
            warn!("Route not found: {}", host.as_ref());
        }
        Ok(())
    }

    // Apply a partial update to an existing route identified by domain (the map key).
    pub async fn update_route(&mut self, domain: &str, patch: RoutePatch) -> Result<()> {
        use log::warn;

        let route = self.routes.get_mut(domain).ok_or_else(|| anyhow::anyhow!(format!("Route not found: {}", domain)))?;

        if let Some(host) = patch.host {
            route.host = host;
        }
        if let Some(path) = patch.path {
            route.path = if path.ends_with('/') {
                let path = trim_trailing_slash(path);
                warn!("Path should not end with '/', will be stripped: {}", path);
                path
            } else {
                path
            };
        }
        if let Some(port) = patch.port {
            if let Err(err) = validate_custom_port(port) {
                return Err(anyhow::anyhow!(err));
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

    // Add a subroute to an existing route
    pub async fn add_subroute(&mut self, domain: &str, path: String, port: u16) -> Result<()> {
        use log::{info, warn};

        let route = self.routes.get_mut(domain).ok_or_else(|| anyhow::anyhow!(format!("Route not found: {}", domain)))?;

        // Validate port
        if let Err(err) = validate_custom_port(port) {
            return Err(anyhow::anyhow!(err));
        }

        // Check if port conflicts with parent route
        if port == route.port {
            return Err(anyhow::anyhow!("Subroute port cannot be the same as the parent route port: {}", port));
        }

        // Clean up path
        let mut clean_path = path;
        if clean_path.ends_with('/') {
            clean_path = trim_trailing_slash(clean_path);
            warn!("Path should not end with '/', will be stripped: {}", clean_path);
        }
        if !clean_path.starts_with('/') {
            clean_path = format!("/{}", clean_path);
            info!("Path should start with '/', prepended: {}", clean_path);
        }

        // Check for duplicate subroute paths
        for existing_subroute in &route.subroutes {
            if existing_subroute.path == clean_path {
                return Err(anyhow::anyhow!("Subroute already exists for path: {}", clean_path));
            }
        }

        let subroute = ProxyPathRoute { path: clean_path.clone(), port };

        route.subroutes.push(subroute);
        info!("Added subroute to {}: {} -> port {}", domain, clean_path, port);
        Ok(())
    }
}

impl ProxyRoute {
    pub fn new(host: String, path: String, port: u16, ssl_enable: bool, listen_port: Option<u16>, redirect_to_https: bool) -> Self {
        Self { host, path, port, ssl_enable, listen_port, redirect_to_https, subroutes: Vec::new() }
    }

    pub fn is_ssl_enabled(&self) -> bool {
        self.ssl_enable
    }

    pub fn get_redirect_to_https(&self) -> bool {
        self.redirect_to_https
    }

    pub fn get_listen_port(&self) -> Option<u16> {
        self.listen_port
    }

    // New getters for the host, port, and path to avoid accessing private fields from other modules
    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string_pretty(self).unwrap();
        writeln!(f, "{}", json)
    }
}

// Helper functions for deserialization
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new("./test_config.json");
        assert_eq!(config.get_email(), "");
        assert_eq!(config.get_cache_dir(), "./cache");
        assert!(config.routes.is_empty());
    }

    #[test]
    fn test_config_set_email() {
        let mut config = Config::default();
        config.set_email("test@example.com".to_string());
        assert_eq!(config.get_email(), "test@example.com");
    }

    #[test]
    fn test_lookup_host_exact_match() {
        let mut config = Config::default();
        config.routes.insert("api.example.com".to_string(), ProxyRoute::new("localhost".to_string(), "/api".to_string(), 8080, false, None, false));

        let route = config.lookup_host("api.example.com");
        assert!(route.is_some());
        assert_eq!(route.unwrap().get_host(), "localhost");
        assert_eq!(route.unwrap().get_port(), 8080);
    }

    #[test]
    fn test_lookup_host_wildcard_match() {
        let mut config = Config::default();
        config.routes.insert("*.example.com".to_string(), ProxyRoute::new("localhost".to_string(), "/".to_string(), 8080, false, None, false));

        // Should match wildcard
        let route = config.lookup_host("api.example.com");
        assert!(route.is_some());
        assert_eq!(route.unwrap().get_host(), "localhost");

        let route2 = config.lookup_host("sub.example.com");
        assert!(route2.is_some());

        // Should not match
        let route3 = config.lookup_host("example.com");
        assert!(route3.is_none());

        let route4 = config.lookup_host("example.org");
        assert!(route4.is_none());
    }

    #[test]
    fn test_lookup_host_exact_over_wildcard() {
        let mut config = Config::default();
        config
            .routes
            .insert("*.example.com".to_string(), ProxyRoute::new("localhost".to_string(), "/wildcard".to_string(), 8080, false, None, false));
        config.routes.insert("api.example.com".to_string(), ProxyRoute::new("localhost".to_string(), "/exact".to_string(), 9090, false, None, false));

        // Exact match should take precedence
        let route = config.lookup_host("api.example.com");
        assert!(route.is_some());
        assert_eq!(route.unwrap().get_path(), "/exact");
        assert_eq!(route.unwrap().get_port(), 9090);
    }

    #[tokio::test]
    async fn test_add_route_success() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/api".to_string(), 8080, true, None, false);
        let result = config.add_route("api.example.com".to_string(), route).await;
        assert!(result.is_ok());
        assert!(config.routes.contains_key("api.example.com"));
    }

    #[tokio::test]
    async fn test_add_route_duplicate() {
        let mut config = Config::default();
        let route1 = ProxyRoute::new("localhost".to_string(), "/api".to_string(), 8080, true, None, false);
        config.add_route("api.example.com".to_string(), route1).await.unwrap();

        let route2 = ProxyRoute::new("localhost".to_string(), "/api".to_string(), 9090, true, None, false);
        let result = config.add_route("api.example.com".to_string(), route2).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_add_route_invalid_port() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/api".to_string(), 80, true, None, false);
        let result = config.add_route("api.example.com".to_string(), route).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("reserved"));
    }

    #[tokio::test]
    async fn test_add_route_trailing_slash() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/api/".to_string(), 8080, true, None, false);
        let result = config.add_route("api.example.com".to_string(), route).await;
        assert!(result.is_ok());
        let added_route = config.lookup_host("api.example.com").unwrap();
        assert_eq!(added_route.get_path(), "/api");
    }

    #[tokio::test]
    async fn test_remove_route() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/api".to_string(), 8080, true, None, false);
        config.add_route("api.example.com".to_string(), route).await.unwrap();

        assert!(config.routes.contains_key("api.example.com"));
        let result = config.remove_route("api.example.com").await;
        assert!(result.is_ok());
        assert!(!config.routes.contains_key("api.example.com"));
    }

    #[tokio::test]
    async fn test_update_route_host() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/api".to_string(), 8080, true, None, false);
        config.add_route("api.example.com".to_string(), route).await.unwrap();

        let patch = RoutePatch { host: Some("127.0.0.1".to_string()), ..Default::default() };
        let result = config.update_route("api.example.com", patch).await;
        assert!(result.is_ok());
        assert_eq!(config.lookup_host("api.example.com").unwrap().get_host(), "127.0.0.1");
    }

    #[tokio::test]
    async fn test_update_route_port() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/api".to_string(), 8080, true, None, false);
        config.add_route("api.example.com".to_string(), route).await.unwrap();

        let patch = RoutePatch { port: Some(9090), ..Default::default() };
        let result = config.update_route("api.example.com", patch).await;
        assert!(result.is_ok());
        assert_eq!(config.lookup_host("api.example.com").unwrap().get_port(), 9090);
    }

    #[tokio::test]
    async fn test_update_route_invalid_port() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/api".to_string(), 8080, true, None, false);
        config.add_route("api.example.com".to_string(), route).await.unwrap();

        let patch = RoutePatch { port: Some(443), ..Default::default() };
        let result = config.update_route("api.example.com", patch).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_route_not_found() {
        let mut config = Config::default();
        let patch = RoutePatch { host: Some("127.0.0.1".to_string()), ..Default::default() };
        let result = config.update_route("nonexistent.example.com", patch).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_add_subroute_success() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/".to_string(), 8080, true, None, false);
        config.add_route("api.example.com".to_string(), route).await.unwrap();

        let result = config.add_subroute("api.example.com", "/metrics".to_string(), 9090).await;
        assert!(result.is_ok());

        let route = config.lookup_host("api.example.com").unwrap();
        assert_eq!(route.subroutes.len(), 1);
        assert_eq!(route.subroutes[0].path, "/metrics");
        assert_eq!(route.subroutes[0].port, 9090);
    }

    #[tokio::test]
    async fn test_add_subroute_prepend_slash() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/".to_string(), 8080, true, None, false);
        config.add_route("api.example.com".to_string(), route).await.unwrap();

        let result = config.add_subroute("api.example.com", "metrics".to_string(), 9090).await;
        assert!(result.is_ok());

        let route = config.lookup_host("api.example.com").unwrap();
        assert_eq!(route.subroutes[0].path, "/metrics");
    }

    #[tokio::test]
    async fn test_add_subroute_duplicate_path() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/".to_string(), 8080, true, None, false);
        config.add_route("api.example.com".to_string(), route).await.unwrap();

        config.add_subroute("api.example.com", "/metrics".to_string(), 9090).await.unwrap();
        let result = config.add_subroute("api.example.com", "/metrics".to_string(), 9091).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_add_subroute_same_port_as_parent() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/".to_string(), 8080, true, None, false);
        config.add_route("api.example.com".to_string(), route).await.unwrap();

        let result = config.add_subroute("api.example.com", "/metrics".to_string(), 8080).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("same as the parent"));
    }

    #[tokio::test]
    async fn test_add_subroute_invalid_port() {
        let mut config = Config::default();
        let route = ProxyRoute::new("localhost".to_string(), "/".to_string(), 8080, true, None, false);
        config.add_route("api.example.com".to_string(), route).await.unwrap();

        let result = config.add_subroute("api.example.com", "/metrics".to_string(), 443).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("reserved"));
    }

    #[test]
    fn test_proxy_route_getters() {
        let route = ProxyRoute::new("localhost".to_string(), "/api/v1".to_string(), 8080, true, Some(8443), true);

        assert_eq!(route.get_host(), "localhost");
        assert_eq!(route.get_path(), "/api/v1");
        assert_eq!(route.get_port(), 8080);
        assert!(route.is_ssl_enabled());
        assert_eq!(route.get_listen_port(), Some(8443));
        assert!(route.get_redirect_to_https());
    }
}
