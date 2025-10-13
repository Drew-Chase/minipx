use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub host: String,
    pub port: i64,
    pub path: String,
    pub ssl_enabled: bool,
    pub redirect_to_https: bool,
    pub listen_port: Option<i64>,
    pub status: String,
    pub binary_path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub domain: String,
    pub host: Option<String>,
    pub port: u16,
    pub path: Option<String>,
    pub ssl_enabled: Option<bool>,
    pub redirect_to_https: Option<bool>,
    pub listen_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: Option<String>,
    pub ssl_enabled: Option<bool>,
    pub redirect_to_https: Option<bool>,
    pub listen_port: Option<u16>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Certificate {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub cert_path: String,
    pub key_path: Option<String>,
    pub is_letsencrypt: bool,
    pub expiry_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCertificateRequest {
    pub name: String,
    pub domain: String,
    pub is_letsencrypt: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceMetric {
    pub id: String,
    pub server_id: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_in: f64,
    pub network_out: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_usage: f64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub network_in: f64,
    pub network_out: f64,
}
