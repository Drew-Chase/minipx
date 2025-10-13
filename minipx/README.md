# Minipx Library

The core library for minipx, a high-performance reverse proxy with automatic SSL certificate management. This library provides modular components for building custom proxy solutions with configuration management, SSL/TLS handling, and HTTP/WebSocket proxying.

## Features

- 🚀 **High Performance**: Built with Tokio for async I/O and excellent performance
- 🔒 **Automatic SSL**: Integrated Let's Encrypt support via ACME (TLS-ALPN-01)
- 🌐 **Multi-Domain**: Route multiple domains with individual SSL certificates
- 🔁 **Smart Routing**: Path-based routing with subroute support
- 🧩 **Hot Reload**: Configuration watching with live updates
- 📊 **Logging**: Structured logging with configurable levels
- 🔌 **Modular**: Use individual components as needed in your own projects

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
minipx = { path = "../minipx" }  # Or version from crates.io when published
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

### Basic Proxy Server

```rust
use minipx::{config::Config, proxy, ssl_server};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::try_load("./minipx.json").await?;

    // Start HTTP and HTTPS servers concurrently
    tokio::try_join!(
        proxy::start_rp_server(),
        ssl_server::start_ssl_server()
    )?;

    Ok(())
}
```

## Examples

The library includes several complete examples demonstrating different use cases and features. You can run any example using:

```bash
cargo run --example <example_name>
```

### Available Examples

#### 1. **basic_proxy** - Simple Proxy Server
The simplest way to create a reverse proxy server.

```bash
cargo run --example basic_proxy
```

Demonstrates:
- Loading configuration from file
- Starting HTTP and HTTPS servers
- Basic logging setup

#### 2. **custom_config** - Programmatic Configuration
Create and configure a proxy server entirely from code.

```bash
cargo run --example custom_config
```

Demonstrates:
- Creating routes programmatically
- Different route types (HTTP, HTTPS, custom ports)
- Wildcard domains
- Saving configuration to disk

#### 3. **route_management** - Dynamic Route Management
Manage routes at runtime: add, update, remove, and query routes.

```bash
cargo run --example route_management
```

Demonstrates:
- Adding multiple routes
- Listing and looking up routes
- Updating routes with partial patches
- Adding subroutes for path-based routing
- Removing routes

#### 4. **hot_reload** - Configuration Hot Reload
Automatic configuration reloading when files change.

```bash
cargo run --example hot_reload
```

Demonstrates:
- Enabling file watching
- Automatic config reload
- Live updates without restart

#### 5. **with_ipc** - IPC Integration
Integrate IPC server for CLI tool communication.

```bash
cargo run --example with_ipc
```

Demonstrates:
- Starting IPC server
- CLI tool integration
- Config path advertisement

#### 6. **advanced_routing** - Advanced Routing Features
Complex routing scenarios with all advanced features.

```bash
cargo run --example advanced_routing
```

Demonstrates:
- Wildcard domains
- Subroutes (path-based routing)
- Custom listen ports
- Mixed HTTP/HTTPS routes
- WebSocket routing
- Microservices backend routing

### Example Files Location

All examples are located in the `examples/` directory:
- `examples/basic_proxy.rs`
- `examples/custom_config.rs`
- `examples/route_management.rs`
- `examples/hot_reload.rs`
- `examples/with_ipc.rs`
- `examples/advanced_routing.rs`

Each example includes detailed comments explaining the code and usage instructions.

## Library Components

### Configuration Management

The `config` module provides configuration loading, validation, and hot-reload capabilities.

#### Loading Configuration

```rust
use minipx::config::Config;

// Create a new default configuration
let config = Config::new("./minipx.json");

// Load from an existing file
let config = Config::try_load("./minipx.json").await?;

// Resolve config path (respects IPC if available)
let path = Config::resolve_config_path(Some("./custom.json".to_string())).await;
```

#### Creating Routes Programmatically

```rust
use minipx::config::{Config, ProxyRoute};

let mut config = Config::new("./minipx.json");

// Create a new route
let route = ProxyRoute::new(
    "localhost".to_string(),    // host
    "/api/v1".to_string(),      // path
    3000,                        // port
    true,                        // ssl_enable
    None,                        // listen_port
    true,                        // redirect_to_https
);

// Add route to configuration
config.add_route("api.example.com".to_string(), route).await?;

// Save configuration
config.save().await?;
```

#### Managing Routes

```rust
use minipx::config::Config;

let mut config = Config::try_load("./minipx.json").await?;

// Get all routes
for (domain, route) in config.get_routes() {
    println!("{} -> {}:{}", domain, route.get_host(), route.get_port());
}

// Lookup a specific route
if let Some(route) = config.lookup_host("api.example.com") {
    println!("Found route: {}:{}/{}",
        route.get_host(),
        route.get_port(),
        route.get_path()
    );
}

// Remove a route
config.remove_route("api.example.com").await?;
config.save().await?;
```

#### Updating Routes

```rust
use minipx::config::{Config, RoutePatch};

let mut config = Config::try_load("./minipx.json").await?;

// Create a partial update
let patch = RoutePatch {
    host: None,
    path: Some("/api/v2".to_string()),
    port: Some(3001),
    ssl_enable: Some(true),
    redirect_to_https: None,
    listen_port: None,
};

// Apply the update
config.update_route("api.example.com", patch).await?;
config.save().await?;
```

#### Adding Subroutes

```rust
use minipx::config::Config;

let mut config = Config::try_load("./minipx.json").await?;

// Add a subroute to an existing domain
config.add_subroute(
    "example.com",       // domain
    "/maps/smp".to_string(),  // path
    8100                 // port
).await?;

config.save().await?;
```

#### Configuration Hot-Reload

```rust
use minipx::config::Config;

let config = Config::try_load("./minipx.json").await?;

// Start watching for configuration changes
// This will automatically reload and apply changes
config.watch_config_file();

// Your application continues running...
// The config will be automatically updated when the file changes
```

### Proxy Server

The `proxy` module provides HTTP and WebSocket reverse proxy functionality.

#### Starting the Proxy Server

```rust
use minipx::proxy;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start the reverse proxy server
    // Listens on configured ports and routes traffic based on Host header
    proxy::start_rp_server().await?;
    Ok(())
}
```

### SSL Server

The `ssl_server` module handles HTTPS with automatic Let's Encrypt certificate management.

#### Starting the SSL Server

```rust
use minipx::ssl_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start the HTTPS server with automatic ACME certificate management
    // Handles TLS-ALPN-01 challenges and certificate renewal
    ssl_server::start_ssl_server().await?;
    Ok(())
}
```

### Inter-Process Communication (IPC)

The `ipc` module provides local socket communication for sharing configuration paths.

#### Starting IPC Server

```rust
use minipx::ipc;
use std::path::PathBuf;

// Start IPC server to advertise config path
let config_path = PathBuf::from("./minipx.json");
ipc::start_ipc_server(config_path);

// This allows CLI tools to discover the running instance's config
```

### Utilities

The `utils` module provides helper functions for path manipulation and validation.

```rust
use minipx::utils::path::trim_trailing_slash;
use minipx::utils::validation::validate_custom_port;

// Trim trailing slashes from paths
let clean_path = trim_trailing_slash("/api/v1/".to_string());
assert_eq!(clean_path, "/api/v1");

// Validate port numbers (rejects 80, 443, and out-of-range)
validate_custom_port(8080)?;  // OK
validate_custom_port(80)?;    // Error: reserved port
```

## Configuration Structure

### Config Object

```rust
pub struct Config {
    email: String,              // ACME email for Let's Encrypt
    cache_dir: String,          // Certificate cache directory
    routes: HashMap<String, ProxyRoute>,  // Domain -> Route mapping
    // ... internal fields
}
```

### ProxyRoute Object

```rust
pub struct ProxyRoute {
    host: String,               // Backend host
    path: String,               // Backend path prefix
    port: u16,                  // Backend port
    ssl_enable: bool,           // Enable SSL for this route
    listen_port: Option<u16>,   // Custom listen port (optional)
    redirect_to_https: bool,    // Redirect HTTP to HTTPS
    subroutes: Vec<ProxyPathRoute>,  // Path-based routing
}
```

### Complete Example

```rust
use minipx::{config::Config, proxy, ssl_server, ipc};
use anyhow::Result;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Load or create configuration
    let config_path = "./minipx.json";
    let config = Config::try_load(config_path).await?;

    // Enable configuration hot-reload
    config.watch_config_file();

    // Start IPC server for CLI integration
    ipc::start_ipc_server(config.get_path().clone());

    info!("Starting minipx proxy servers...");

    // Run both HTTP and HTTPS servers concurrently
    tokio::try_join!(
        proxy::start_rp_server(),
        ssl_server::start_ssl_server()
    )?;

    Ok(())
}
```

## Configuration File Format

The library uses JSON configuration files:

```json
{
  "email": "admin@example.com",
  "cache_dir": "./cache",
  "routes": {
    "api.example.com": {
      "host": "localhost",
      "path": "/api/v1",
      "port": 3000,
      "ssl_enable": true,
      "redirect_to_https": true,
      "subroutes": [
        {
          "path": "/api/v1/legacy",
          "port": 3001
        }
      ]
    },
    "*.example.com": {
      "host": "192.168.1.100",
      "path": "",
      "port": 8080,
      "ssl_enable": false,
      "redirect_to_https": false
    }
  }
}
```

## Advanced Usage

### Custom Server Implementation

```rust
use minipx::config::{Config, ProxyRoute};
use std::collections::HashMap;

// Build configuration programmatically
let mut routes = HashMap::new();

routes.insert(
    "api.example.com".to_string(),
    ProxyRoute::new(
        "localhost".to_string(),
        "/api".to_string(),
        3000,
        true,
        None,
        true,
    ),
);

// Create config with custom settings
let mut config = Config::new("./custom.json");
config.set_email("admin@example.com".to_string());

// Add routes
for (domain, route) in routes {
    config.add_route(domain, route).await?;
}

// Save and use
config.save().await?;
```

### Wildcard Domain Support

```rust
use minipx::config::{Config, ProxyRoute};

let mut config = Config::new("./minipx.json");

// Wildcard domains are supported for routing lookups
// Note: Wildcard SSL certificates are NOT automatically generated
let route = ProxyRoute::new(
    "localhost".to_string(),
    "".to_string(),
    8080,
    false,
    None,
    false,
);

config.add_route("*.example.com".to_string(), route).await?;

// Lookup works with wildcard matching
let route = config.lookup_host("subdomain.example.com");
assert!(route.is_some());
```

## API Reference

### Config Methods

- `new(path: impl AsRef<Path>) -> Self` - Create new config
- `try_load(path: impl AsRef<Path>) -> Result<Self>` - Load from file
- `save() -> Result<()>` - Save configuration to file
- `watch_config_file()` - Enable hot-reload
- `add_route(domain: String, route: ProxyRoute) -> Result<()>` - Add route
- `remove_route(host: &str) -> Result<()>` - Remove route
- `update_route(domain: &str, patch: RoutePatch) -> Result<()>` - Update route
- `add_subroute(domain: &str, path: String, port: u16) -> Result<()>` - Add subroute
- `lookup_host(key: &str) -> Option<&ProxyRoute>` - Find route by domain
- `get_routes() -> &HashMap<String, ProxyRoute>` - Get all routes
- `set_email(email: String)` - Set ACME email
- `get_email() -> &String` - Get ACME email
- `get_cache_dir() -> &String` - Get cache directory
- `get_path() -> &PathBuf` - Get config file path

### ProxyRoute Methods

- `new(host, path, port, ssl_enable, listen_port, redirect_to_https) -> Self` - Create route
- `get_host() -> &str` - Get backend host
- `get_port() -> u16` - Get backend port
- `get_path() -> &str` - Get backend path
- `is_ssl_enabled() -> bool` - Check if SSL is enabled
- `get_redirect_to_https() -> bool` - Check redirect setting
- `get_listen_port() -> Option<u16>` - Get custom listen port

## Dependencies

- `tokio` - Async runtime
- `hyper` - HTTP implementation
- `rustls` - TLS implementation
- `rustls-acme` - Let's Encrypt ACME integration
- `serde` / `serde_json` - Serialization
- `anyhow` - Error handling
- `log` - Logging facade
- `notify` - File watching for hot-reload
- `interprocess` - IPC communication

## Thread Safety

All configuration operations are designed to work in multi-threaded async contexts. The `Config` type uses internal synchronization for safe concurrent access during hot-reload operations.

## Error Handling

The library uses `anyhow::Result` for error handling. All public APIs return `Result` types for proper error propagation:

```rust
use anyhow::Result;

async fn example() -> Result<()> {
    let config = Config::try_load("./config.json").await?;
    config.add_route("example.com".to_string(), route).await?;
    Ok(())
}
```

## Contributing

See the main [minipx repository](../) for contribution guidelines.

## License

MIT License - See [LICENSE](../LICENSE) file for details.
