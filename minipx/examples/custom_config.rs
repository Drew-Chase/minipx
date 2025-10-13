//! Custom Configuration Example
//!
//! This example demonstrates how to create and configure a minipx proxy server
//! entirely from code, without needing a configuration file.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example custom_config
//! ```

use minipx::config::{Config, ProxyRoute};
use anyhow::Result;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Creating custom configuration");

    // Create a new configuration
    let mut config = Config::new("./custom-proxy.json");

    // Set ACME email for Let's Encrypt
    config.set_email("admin@example.com".to_string());

    info!("Adding routes...");

    // Add a simple HTTP route for a development API
    let api_route = ProxyRoute::new(
        "localhost".to_string(),     // Backend host
        "/api/v1".to_string(),       // Backend path
        3000,                        // Backend port
        false,                       // SSL disabled (development)
        None,                        // No custom listen port
        false,                       // No HTTP->HTTPS redirect
    );
    config.add_route("api.localhost".to_string(), api_route).await?;
    info!("Added route: api.localhost -> localhost:3000/api/v1");

    // Add an HTTPS route for a production web app
    let web_route = ProxyRoute::new(
        "192.168.1.100".to_string(), // Backend host
        "".to_string(),               // No path prefix
        8080,                        // Backend port
        true,                        // SSL enabled
        None,                        // Use default HTTPS port (443)
        true,                        // Redirect HTTP to HTTPS
    );
    config.add_route("app.example.com".to_string(), web_route).await?;
    info!("Added route: app.example.com -> 192.168.1.100:8080 (HTTPS with redirect)");

    // Add a wildcard route for subdomains
    let wildcard_route = ProxyRoute::new(
        "10.0.0.50".to_string(),     // Backend host
        "".to_string(),               // No path prefix
        9000,                        // Backend port
        false,                       // SSL disabled
        None,                        // Default port
        false,                       // No redirect
    );
    config.add_route("*.dev.local".to_string(), wildcard_route).await?;
    info!("Added wildcard route: *.dev.local -> 10.0.0.50:9000");

    // Add a route with custom listen port (e.g., for game servers)
    let game_route = ProxyRoute::new(
        "192.168.1.200".to_string(), // Backend host
        "".to_string(),               // No path prefix
        7777,                        // Backend port
        false,                       // SSL not applicable for custom ports
        Some(25565),                 // Custom listen port (Minecraft default)
        false,                       // No redirect
    );
    config.add_route("game.example.com".to_string(), game_route).await?;
    info!("Added custom port route: game.example.com:25565 -> 192.168.1.200:7777");

    // Save the configuration to disk
    config.save().await?;
    info!("Configuration saved to: {}", config.get_path().display());

    // Display the final configuration
    println!("\nConfiguration created successfully!");
    println!("{}", config);

    Ok(())
}
