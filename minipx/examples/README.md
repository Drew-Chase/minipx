# Minipx Library Examples

This directory contains complete, runnable examples demonstrating various features and use cases of the minipx library.

## Running Examples

You can run any example from the workspace root using:

```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example basic_proxy
```

## Available Examples

### 1. Basic Proxy Server (`basic_proxy.rs`)

**The simplest way to create a reverse proxy server.**

```bash
cargo run --example basic_proxy
```

**What it demonstrates:**
- Loading configuration from a JSON file
- Starting HTTP and HTTPS servers concurrently
- Basic logging configuration
- Minimal setup for a working proxy

**Best for:** Getting started quickly, understanding the basics

---

### 2. Custom Configuration (`custom_config.rs`)

**Create and configure a proxy server entirely from code without needing a config file.**

```bash
cargo run --example custom_config
```

**What it demonstrates:**
- Creating a `Config` object programmatically
- Adding different types of routes (HTTP, HTTPS, custom ports)
- Setting up wildcard domain routes
- Configuring ACME email
- Saving configuration to disk

**Best for:** Applications that need to generate configurations dynamically

---

### 3. Route Management (`route_management.rs`)

**Comprehensive example of managing routes at runtime.**

```bash
cargo run --example route_management
```

**What it demonstrates:**
- Adding multiple routes programmatically
- Listing all configured routes
- Looking up specific routes by domain
- Wildcard domain matching
- Updating routes with partial patches (`RoutePatch`)
- Adding subroutes for path-based routing
- Removing routes
- Complete CRUD operations on routes

**Best for:** Applications with dynamic routing requirements, admin interfaces

---

### 4. Hot Reload (`hot_reload.rs`)

**Automatic configuration reloading when the config file changes.**

```bash
cargo run --example hot_reload
```

**What it demonstrates:**
- Enabling file watching with `watch_config_file()`
- Automatic configuration reload on file changes
- Live updates without server restart
- HTTPS server restart when SSL settings change

**Usage:**
1. Run the example
2. Modify the generated config file
3. Watch the logs as changes are automatically applied

**Best for:** Development environments, production setups requiring zero-downtime config updates

---

### 5. IPC Integration (`with_ipc.rs`)

**Integrate IPC server for CLI tool communication.**

```bash
cargo run --example with_ipc
```

**What it demonstrates:**
- Starting the IPC server with `ipc::start_ipc_server()`
- Config path advertisement to CLI tools
- Enabling CLI management of running instances
- How IPC enables the CLI to discover running instances

**Usage:**
1. Run the example
2. In another terminal, use minipx CLI commands without specifying config
3. The CLI will automatically discover and use this instance's config

**Best for:** Production deployments, multi-instance management

---

### 6. Advanced Routing (`advanced_routing.rs`)

**Comprehensive demonstration of all advanced routing features.**

```bash
cargo run --example advanced_routing
```

**What it demonstrates:**
- Wildcard domain routing (`*.dev.example.com`)
- Multiple subroutes per domain (path-based routing)
- Custom listen ports for specialized services
- Mixed HTTP and HTTPS routes
- WebSocket server routing (WSS)
- Microservices backend with multiple subroutes
- Game server routing on custom ports
- Complete real-world routing scenarios

**Best for:** Understanding complex routing scenarios, production configurations

---

## Example Structure

Each example follows this structure:

1. **File header** - Documentation comment explaining the example
2. **Usage instructions** - How to run the example
3. **Implementation** - Well-commented code
4. **Helper functions** - Supporting functions for the example (where needed)

## Common Patterns

### Basic Server Setup

```rust
use minipx::{config::Config, proxy, ssl_server};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::try_load("./minipx.json").await?;

    tokio::try_join!(
        proxy::start_rp_server(),
        ssl_server::start_ssl_server()
    )?;

    Ok(())
}
```

### Creating Routes Programmatically

```rust
use minipx::config::{Config, ProxyRoute};

let mut config = Config::new("./config.json");

let route = ProxyRoute::new(
    "localhost".to_string(),
    "/api".to_string(),
    3000,
    true,   // SSL enabled
    None,   // Default port
    true,   // Redirect to HTTPS
);

config.add_route("api.example.com".to_string(), route).await?;
config.save().await?;
```

### Updating Routes

```rust
use minipx::config::RoutePatch;

let patch = RoutePatch {
    host: None,                         // Keep existing
    path: Some("/api/v2".to_string()),  // Update path
    port: Some(3001),                   // Update port
    ssl_enable: None,                   // Keep existing
    redirect_to_https: Some(false),     // Disable redirect
    listen_port: None,                  // Keep existing
};

config.update_route("api.example.com", patch).await?;
```

### Adding Subroutes

```rust
// Add path-based routing under a domain
config.add_subroute(
    "api.example.com",
    "/v1".to_string(),
    3001
).await?;

// Request to api.example.com/v1/users -> localhost:3001/users
// (The /v1 prefix is stripped)
```

## Tips

1. **Start with `basic_proxy.rs`** to understand the minimal setup
2. **Use `custom_config.rs`** to learn programmatic configuration
3. **Run `route_management.rs`** to understand route CRUD operations
4. **Try `hot_reload.rs`** to see live configuration updates in action
5. **Study `advanced_routing.rs`** for complex production scenarios

## Dependencies

All examples require:
- Rust 1.70+
- Tokio runtime
- The minipx library

Dependencies are automatically handled by Cargo when running examples from the workspace.

## Development

To create a new example:

1. Create a new `.rs` file in this directory
2. Add documentation comments explaining the example
3. Implement the example with clear comments
4. Add it to the list in this README
5. Test it: `cargo run --example your_example`

## See Also

- [Library README](../README.md) - Complete library documentation
- [CLI README](../../cli/README.MD) - Command-line interface usage
- [Main README](../../README.MD) - Project overview
