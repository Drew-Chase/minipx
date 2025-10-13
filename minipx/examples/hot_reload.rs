//! Hot Reload Example
//!
//! This example demonstrates configuration hot-reloading functionality.
//! The proxy server will automatically detect and apply configuration changes
//! without requiring a restart.
//!
//! # Usage
//!
//! ```bash
//! # In one terminal, run the example
//! cargo run --example hot_reload
//!
//! # In another terminal, modify the configuration file
//! # The server will automatically detect and apply changes
//! ```

use anyhow::Result;
use minipx::{config::Config, proxy, ssl_server};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting minipx with hot-reload enabled");

    // Create a configuration file with some initial routes
    let config_path = "./hot-reload-example.json";
    setup_initial_config(config_path).await?;

    // Load the configuration
    let config = Config::try_load(config_path).await?;

    println!("Configuration loaded from: {}", config_path);
    println!("Initial routes: {}", config.get_routes().len());

    // Enable configuration file watching
    // This starts a background task that monitors the file for changes
    config.watch_config_file();

    println!("Hot-reload enabled! The server will automatically apply configuration changes.");
    println!("Try modifying {} to see hot-reload in action", config_path);

    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║           HOT RELOAD DEMONSTRATION                       ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║                                                           ║");
    println!("║  The proxy server is now running with hot-reload enabled ║");
    println!("║                                                           ║");
    println!("║  Try these experiments:                                  ║");
    println!("║                                                           ║");
    println!("║  1. Add a new route to the config file                   ║");
    println!("║     → Server will automatically start routing to it      ║");
    println!("║                                                           ║");
    println!("║  2. Modify an existing route's port or host              ║");
    println!("║     → Server will update the backend target              ║");
    println!("║                                                           ║");
    println!("║  3. Change ssl_enable to true for a route                ║");
    println!("║     → HTTPS server will restart to provision certs       ║");
    println!("║                                                           ║");
    println!("║  4. Update the ACME email address                        ║");
    println!("║     → HTTPS server will restart with new account         ║");
    println!("║                                                           ║");
    println!("║  5. Delete a route from the config                       ║");
    println!("║     → Server will stop routing to that domain            ║");
    println!("║                                                           ║");
    println!("║  Config file: {}                           ║", config_path);
    println!("║                                                           ║");
    println!("║  Press Ctrl+C to stop the server                         ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Start the proxy servers
    // Note: Configuration changes will be automatically picked up
    // by the running servers without requiring a restart
    tokio::try_join!(proxy::start_rp_server(), ssl_server::start_ssl_server())?;

    Ok(())
}

/// Set up an initial configuration file for the hot-reload demonstration
async fn setup_initial_config(config_path: &str) -> Result<()> {
    use minipx::config::ProxyRoute;

    println!("Creating initial configuration");

    let mut config = Config::new(config_path);
    config.set_email("admin@example.com".to_string());

    // Add some initial routes
    config.add_route("api.localhost".to_string(), ProxyRoute::new("localhost".to_string(), "/api".to_string(), 3000, false, None, false)).await?;

    config.add_route("web.localhost".to_string(), ProxyRoute::new("localhost".to_string(), "".to_string(), 8080, false, None, false)).await?;

    config.save().await?;
    println!("Initial configuration saved");

    Ok(())
}
