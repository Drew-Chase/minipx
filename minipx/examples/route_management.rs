//! Route Management Example
//!
//! This example demonstrates how to programmatically manage routes:
//! adding, updating, removing, and querying routes from code.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example route_management
//! ```

use anyhow::Result;
use minipx::config::{Config, ProxyRoute, RoutePatch};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Route Management Example");

    // Create or load configuration
    let mut config = Config::new("./route-example.json");
    config.set_email("admin@example.com".to_string());

    println!("\n=== Adding Routes ===");

    // Add multiple routes
    let routes = vec![
        ("api.example.com", ProxyRoute::new("localhost".to_string(), "/api".to_string(), 3000, true, None, true)),
        ("web.example.com", ProxyRoute::new("localhost".to_string(), "".to_string(), 8080, true, None, true)),
        ("admin.example.com", ProxyRoute::new("localhost".to_string(), "/admin".to_string(), 9000, true, None, false)),
    ];

    for (domain, route) in routes {
        config.add_route(domain.to_string(), route).await?;
        println!("✓ Added route: {}", domain);
    }

    println!("\n=== Listing All Routes ===");

    // List all routes
    for (domain, route) in config.get_routes() {
        println!(
            "• {} -> {}:{}/{}{}",
            domain,
            route.get_host(),
            route.get_port(),
            route.get_path(),
            if route.is_ssl_enabled() { " [SSL]" } else { "" }
        );
    }

    println!("\n=== Looking Up Specific Route ===");

    // Lookup a specific route by domain
    if let Some(route) = config.lookup_host("api.example.com") {
        println!("Found route for api.example.com:");
        println!("  Host: {}", route.get_host());
        println!("  Port: {}", route.get_port());
        println!("  Path: /{}", route.get_path());
        println!("  SSL: {}", route.is_ssl_enabled());
        println!("  Redirect: {}", route.get_redirect_to_https());
    }

    // Wildcard lookup example
    config.add_route("*.dev.example.com".to_string(), ProxyRoute::new("localhost".to_string(), "".to_string(), 4000, false, None, false)).await?;

    if let Some(route) = config.lookup_host("subdomain.dev.example.com") {
        println!("\nWildcard match for subdomain.dev.example.com:");
        println!("  Matched route: *.dev.example.com");
        println!("  Backend: {}:{}", route.get_host(), route.get_port());
    }

    println!("\n=== Updating a Route ===");

    // Update a route using RoutePatch (partial update)
    let patch = RoutePatch {
        host: None,                        // Keep existing host
        path: Some("/api/v2".to_string()), // Update path
        port: Some(3001),                  // Update port
        ssl_enable: None,                  // Keep existing SSL setting
        redirect_to_https: Some(false),    // Disable redirect
        listen_port: None,                 // Keep existing listen port
    };

    config.update_route("api.example.com", patch).await?;
    println!("✓ Updated api.example.com");

    if let Some(route) = config.lookup_host("api.example.com") {
        println!("  New path: /{}", route.get_path());
        println!("  New port: {}", route.get_port());
        println!("  Redirect: {}", route.get_redirect_to_https());
    }

    println!("\n=== Adding Subroutes ===");

    // Add subroutes for path-based routing
    config.add_subroute("web.example.com", "/api".to_string(), 3000).await?;
    println!("✓ Added subroute: web.example.com/api -> port 3000");

    config.add_subroute("web.example.com", "/static".to_string(), 8081).await?;
    println!("✓ Added subroute: web.example.com/static -> port 8081");

    // Demonstrate subroute structure
    if let Some(route) = config.lookup_host("web.example.com") {
        println!("\nSubroutes for web.example.com:");
        // Note: subroutes field is private, this is for demonstration
        // In real code, you'd typically just configure and let the proxy handle it
        println!("  Main: {}:{}", route.get_host(), route.get_port());
        println!("  /api -> port 3000 (strips /api prefix)");
        println!("  /static -> port 8081 (strips /static prefix)");
    }

    println!("\n=== Removing a Route ===");

    config.remove_route("admin.example.com").await?;
    println!("✓ Removed route: admin.example.com");

    println!("\n=== Final Route Count ===");
    println!("Total routes: {}", config.get_routes().len());

    // Save configuration
    config.save().await?;
    println!("\n✓ Configuration saved to: {}", config.get_path().display());

    println!("\n=== Configuration Summary ===");
    println!("{}", config);

    Ok(())
}
