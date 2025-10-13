//! IPC Integration Example
//!
//! This example demonstrates how to integrate the IPC server for
//! communication between the proxy instance and CLI tools.
//!
//! # Usage
//!
//! ```bash
//! # Start the proxy with IPC
//! cargo run --example with_ipc
//!
//! # In another terminal, you can use the CLI to manage this instance
//! # The CLI will automatically discover the running instance via IPC
//! ```

use minipx::{config::Config, ipc, proxy, ssl_server};
use anyhow::Result;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting minipx with IPC support");

    // Load or create configuration
    let config_path = "./minipx-with-ipc.json";
    let config = setup_config(config_path).await?;

    info!("Configuration loaded from: {}", config_path);

    // Start the IPC server
    // This allows CLI tools to discover the running instance and its config path
    // The IPC server runs on a local socket and is not exposed over the network
    ipc::start_ipc_server(config.get_path().clone());

    info!("IPC server started - CLI tools can now discover this instance");

    // Enable hot-reload for configuration changes
    config.watch_config_file();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║              MINIPX WITH IPC SUPPORT                      ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║                                                            ║");
    println!("║  The proxy is running with IPC server enabled             ║");
    println!("║                                                            ║");
    println!("║  You can now use CLI commands without specifying config:  ║");
    println!("║                                                            ║");
    println!("║    minipx routes list                                     ║");
    println!("║    minipx routes add example.com --port 8080              ║");
    println!("║    minipx routes show example.com                         ║");
    println!("║    minipx config show                                     ║");
    println!("║                                                            ║");
    println!("║  The CLI will automatically use the config from this      ║");
    println!("║  running instance via IPC.                                ║");
    println!("║                                                            ║");
    println!("║  Config file: {}            ║", config_path);
    println!("║                                                            ║");
    println!("║  Press Ctrl+C to stop                                     ║");
    println!("║                                                            ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Start the proxy servers
    tokio::try_join!(
        proxy::start_rp_server(),
        ssl_server::start_ssl_server()
    )?;

    Ok(())
}

/// Set up configuration for the IPC example
async fn setup_config(config_path: &str) -> Result<Config> {
    use minipx::config::ProxyRoute;

    let mut config = Config::new(config_path);
    config.set_email("admin@example.com".to_string());

    // Add some example routes
    config.add_route(
        "api.example.com".to_string(),
        ProxyRoute::new(
            "localhost".to_string(),
            "/api".to_string(),
            3000,
            false,
            None,
            false,
        ),
    ).await?;

    config.add_route(
        "web.example.com".to_string(),
        ProxyRoute::new(
            "localhost".to_string(),
            "".to_string(),
            8080,
            false,
            None,
            false,
        ),
    ).await?;

    config.save().await?;

    Config::try_load(config_path).await
}
