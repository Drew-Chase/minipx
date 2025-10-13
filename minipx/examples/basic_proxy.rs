//! Basic Proxy Server Example
//!
//! This example demonstrates the simplest way to use the minipx library
//! to create a reverse proxy server with automatic SSL certificate management.
//!
//! # Usage
//!
//! ```bash
//! # Create a configuration file first
//! cargo run --example basic_proxy
//! ```
//!
//! The proxy will start and serve requests based on the configuration file.

use anyhow::Result;
use minipx::{config::Config, proxy, ssl_server};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting basic minipx proxy server");

    // Load configuration from file
    // This will use the default location or create a new config if it doesn't exist
    let config_path = "./minipx.json";
    let config = Config::try_load(config_path).await?;

    println!("Loaded configuration from: {}", config_path);
    println!("Email: {}", config.get_email());
    println!("Cache directory: {}", config.get_cache_dir());
    println!("Number of routes: {}", config.get_routes().len());

    // Start both HTTP and HTTPS servers concurrently
    // The proxy will route traffic based on the Host header
    // The SSL server will automatically handle ACME challenges and certificate provisioning
    println!("Starting HTTP and HTTPS servers...");
    tokio::try_join!(proxy::start_rp_server(), ssl_server::start_ssl_server())?;

    Ok(())
}
