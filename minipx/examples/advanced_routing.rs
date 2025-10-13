//! Advanced Routing Example
//!
//! This example demonstrates advanced routing features:
//! - Wildcard domains
//! - Subroutes (path-based routing)
//! - Custom listen ports
//! - Mixed HTTP/HTTPS routes
//!
//! # Usage
//!
//! ```bash
//! cargo run --example advanced_routing
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

    info!("Advanced Routing Configuration Example");

    let mut config = Config::new("./advanced-routing.json");
    config.set_email("admin@example.com".to_string());

    println!("\n=== Setting Up Advanced Routing ===\n");

    // 1. Main application with HTTPS
    println!("1. Main Application (HTTPS)");
    let main_route = ProxyRoute::new(
        "localhost".to_string(),
        "".to_string(),
        8080,
        true,  // SSL enabled
        None,  // Default HTTPS port (443)
        true,  // Redirect HTTP to HTTPS
    );
    config.add_route("example.com".to_string(), main_route).await?;
    println!("   ✓ example.com -> localhost:8080 [HTTPS with redirect]");

    // 2. API with path-based subrouting
    println!("\n2. API with Subroutes (Path-based routing)");
    let api_route = ProxyRoute::new(
        "localhost".to_string(),
        "/api".to_string(),
        3000,  // Default API backend
        true,
        None,
        true,
    );
    config.add_route("api.example.com".to_string(), api_route).await?;
    println!("   ✓ api.example.com -> localhost:3000/api [HTTPS]");

    // Add subroutes for different API versions
    config.add_subroute("api.example.com", "/v1".to_string(), 3001).await?;
    println!("   ✓ api.example.com/v1 -> localhost:3001 (strips /v1)");

    config.add_subroute("api.example.com", "/v2".to_string(), 3002).await?;
    println!("   ✓ api.example.com/v2 -> localhost:3002 (strips /v2)");

    config.add_subroute("api.example.com", "/admin".to_string(), 3003).await?;
    println!("   ✓ api.example.com/admin -> localhost:3003 (strips /admin)");

    // 3. Wildcard domain for development subdomains
    println!("\n3. Wildcard Development Domains");
    let dev_route = ProxyRoute::new(
        "localhost".to_string(),
        "".to_string(),
        4000,
        false,  // No SSL for dev environments
        None,
        false,
    );
    config.add_route("*.dev.example.com".to_string(), dev_route).await?;
    println!("   ✓ *.dev.example.com -> localhost:4000");
    println!("     Matches: app.dev.example.com, test.dev.example.com, etc.");

    // 4. Static file server with subroutes
    println!("\n4. Static File Server with Media Subroutes");
    let static_route = ProxyRoute::new(
        "localhost".to_string(),
        "".to_string(),
        8081,
        true,
        None,
        false,
    );
    config.add_route("static.example.com".to_string(), static_route).await?;
    println!("   ✓ static.example.com -> localhost:8081 [HTTPS]");

    config.add_subroute("static.example.com", "/images".to_string(), 8082).await?;
    println!("   ✓ static.example.com/images -> localhost:8082");

    config.add_subroute("static.example.com", "/videos".to_string(), 8083).await?;
    println!("   ✓ static.example.com/videos -> localhost:8083");

    // 5. Custom port for game server
    println!("\n5. Game Server (Custom Port)");
    let game_route = ProxyRoute::new(
        "192.168.1.100".to_string(),
        "".to_string(),
        7777,     // Backend game server port
        false,    // No SSL for custom ports
        Some(25565),  // Listen on Minecraft default port
        false,
    );
    config.add_route("game.example.com".to_string(), game_route).await?;
    println!("   ✓ game.example.com:25565 -> 192.168.1.100:7777");

    // 6. WebSocket server
    println!("\n6. WebSocket Server");
    let ws_route = ProxyRoute::new(
        "localhost".to_string(),
        "/ws".to_string(),
        5000,
        true,  // WSS (WebSocket over SSL)
        None,
        true,
    );
    config.add_route("ws.example.com".to_string(), ws_route).await?;
    println!("   ✓ ws.example.com -> localhost:5000/ws [WSS]");

    // 7. Load balancer backend (multiple services)
    println!("\n7. Microservices Backend");
    let services_route = ProxyRoute::new(
        "localhost".to_string(),
        "".to_string(),
        9000,  // Default service
        true,
        None,
        true,
    );
    config.add_route("services.example.com".to_string(), services_route).await?;
    println!("   ✓ services.example.com -> localhost:9000 [HTTPS]");

    // Add microservice subroutes
    config.add_subroute("services.example.com", "/auth".to_string(), 9001).await?;
    println!("   ✓ services.example.com/auth -> localhost:9001");

    config.add_subroute("services.example.com", "/users".to_string(), 9002).await?;
    println!("   ✓ services.example.com/users -> localhost:9002");

    config.add_subroute("services.example.com", "/payments".to_string(), 9003).await?;
    println!("   ✓ services.example.com/payments -> localhost:9003");

    // 8. Admin panel (HTTP only, internal)
    println!("\n8. Admin Panel (Internal, HTTP only)");
    let admin_route = ProxyRoute::new(
        "10.0.0.10".to_string(),
        "/admin".to_string(),
        3000,
        false,  // No SSL for internal admin
        Some(8888),  // Custom port for admin access
        false,
    );
    config.add_route("admin.internal".to_string(), admin_route).await?;
    println!("   ✓ admin.internal:8888 -> 10.0.0.10:3000/admin");

    // Save configuration
    config.save().await?;

    println!("\n=== Configuration Summary ===\n");
    println!("Total domains configured: {}", config.get_routes().len());
    println!("Configuration saved to: {}\n", config.get_path().display());

    // Print the full configuration as JSON
    println!("=== Full Configuration (JSON) ===\n");
    println!("{}", config);

    println!("\n=== Routing Examples ===\n");
    println!("HTTP Requests:");
    println!("  http://example.com → redirects to https://example.com");
    println!("  http://api.dev.example.com → localhost:4000 (wildcard match)");
    println!();
    println!("HTTPS Requests:");
    println!("  https://example.com → localhost:8080");
    println!("  https://api.example.com/v1/users → localhost:3001/users (strips /v1)");
    println!("  https://api.example.com/v2/users → localhost:3002/users (strips /v2)");
    println!("  https://static.example.com/images/logo.png → localhost:8082/logo.png (strips /images)");
    println!();
    println!("Custom Ports:");
    println!("  game.example.com:25565 → 192.168.1.100:7777");
    println!("  admin.internal:8888 → 10.0.0.10:3000/admin");
    println!();
    println!("WebSockets:");
    println!("  wss://ws.example.com → localhost:5000/ws");
    println!();

    Ok(())
}
