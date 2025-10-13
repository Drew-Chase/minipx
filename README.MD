# Minipx

A fast, lightweight, and configurable reverse proxy written in Rust with automatic SSL certificate management via Let's Encrypt.

## Overview

Minipx is a modern reverse proxy solution designed for simplicity and performance. It provides automatic HTTPS with Let's Encrypt, multi-domain routing, hot configuration reloading, and a powerful CLI for management. Built with Rust and Tokio, it offers excellent performance and memory safety.

## Features

- 🚀 **High Performance**: Built with Rust and Tokio for async I/O
- 🔒 **Automatic SSL**: Integrated Let's Encrypt with ACME (TLS-ALPN-01)
- 🌐 **Multi-Domain**: Route multiple domains with individual SSL certificates
- 🔁 **Smart Redirects**: Optional per-route HTTP→HTTPS redirects
- 🧩 **Hot Reload**: Live configuration updates without downtime
- 🛠️ **CLI Management**: Full-featured command-line interface
- 📊 **Structured Logging**: Configurable log levels for monitoring
- 🔌 **Modular Library**: Use as a library in your own projects
- 🎯 **Path-Based Routing**: Subroutes for complex routing scenarios
- 🔄 **WebSocket Support**: Full WebSocket proxying capability

## Project Structure

Minipx is organized as a workspace with multiple components:

```
minipx/
├── cli/          # Command-line interface application
├── minipx/       # Core library (proxy, SSL, config management)
├── web/          # Web-based management interface (optional)
└── tools/        # Build and deployment tools
```

## Quick Start

### For End Users (CLI)

If you want to use minipx as a reverse proxy:

**Installation:**
```bash
# Linux (easy install)
sudo bash -c "$(curl -sSL https://raw.githubusercontent.com/Drew-Chase/minipx/master/cli/install.sh)" < /dev/tty

# Or build from source
git clone <repository-url>
cd minipx
cargo build --release -p minipx_cli
```

**Basic Usage:**
```bash
# Run with auto-detected config
minipx

# With custom config and hot-reload
minipx --watch --config ./minipx.json

# Manage routes via CLI
minipx routes add api.example.com --host 127.0.0.1 --port 8080 --ssl
```

**📖 Full Documentation**: [CLI README](cli/README.MD)

### For Developers (Library)

If you want to integrate minipx into your own Rust project:

**Add to Cargo.toml:**
```toml
[dependencies]
minipx = { path = "./minipx" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

**Example Usage:**
```rust
use minipx::{config::Config, proxy, ssl_server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::try_load("./minipx.json").await?;
    config.watch_config_file();

    tokio::try_join!(
        proxy::start_rp_server(),
        ssl_server::start_ssl_server()
    )?;

    Ok(())
}
```

**📖 Full Documentation**: [Library README](minipx/README.md)

## Video Tutorial

Watch a comprehensive tutorial on setting up and using minipx:

<a href="https://youtu.be/2Uo7rTiK_h0" target="_blank" rel="noopener noreferrer">
  <img src="https://img.youtube.com/vi/2Uo7rTiK_h0/maxresdefault.jpg" alt="Minipx tutorial on YouTube" width="640" />
</a>

## Configuration

Minipx uses JSON configuration files for defining routes, SSL settings, and proxy behavior.

### Example Configuration

```json
{
  "email": "admin@example.com",
  "cache_dir": "./ssl-cache",
  "routes": {
    "api.example.com": {
      "host": "localhost",
      "path": "/api/v1",
      "port": 3000,
      "ssl_enable": true,
      "redirect_to_https": true
    },
    "*.example.com": {
      "host": "192.168.1.100",
      "port": 8080,
      "ssl_enable": true,
      "redirect_to_https": true
    }
  }
}
```

**Key Features:**
- **Wildcard domains**: Use `*.example.com` for subdomain matching
- **Subroutes**: Path-based routing to different backends
- **Custom ports**: Per-route listen ports for specialized services
- **Hot reload**: Changes apply automatically with `--watch`

For detailed configuration options, see:
- [Library Documentation](minipx/README.md) - Configuration API and structure
- [CLI Documentation](cli/README.MD) - CLI-based configuration management

## How It Works

### Server Architecture

- **HTTP Server** (port 80): Handles plain HTTP traffic and redirects
- **HTTPS Server** (port 443): TLS termination with automatic certificates
- **Custom Listeners**: Additional ports per route for specialized services
- **IPC Server**: Local socket for CLI communication

### SSL Certificate Management

- **Automatic Provisioning**: Certificates requested from Let's Encrypt
- **TLS-ALPN-01 Validation**: Served on port 443
- **Multi-Domain**: Individual certificates per domain
- **Auto-Renewal**: Handled transparently by rustls-acme
- **Caching**: Certificates cached in `cache_dir` to avoid rate limits

### Routing Logic

1. Request arrives on configured port
2. Host header determines the route
3. Path matching checks for subroutes
4. Request proxied to backend with path rewriting
5. WebSocket upgrades handled automatically

## Use Cases

### Web Application Hosting
Route multiple web applications on different domains with automatic SSL.

### Microservices Gateway
Direct traffic to different microservices based on path or domain.

### Development Proxy
Hot-reload configuration for rapid development iteration.

### Game Server Proxy
Custom port routing for game servers and specialized protocols.

### API Gateway
Path-based routing with SSL termination for API backends.

## Common CLI Commands

```bash
# Server management
minipx                                    # Start with default config
minipx --watch --verbose                  # Start with hot-reload and logging

# Route management
minipx routes list                        # List all routes
minipx routes add example.com --port 8080 # Add new route
minipx routes update example.com --ssl    # Enable SSL for route
minipx routes remove example.com          # Remove route

# Configuration
minipx config show                        # Display current config
minipx config email admin@example.com     # Set ACME email
```

See [CLI README](cli/README.MD) for complete command reference.

## Requirements

### For HTTP (Port 80)
- Ability to bind to port 80 (may require sudo/capabilities)
- Backend services running and accessible

### For HTTPS (Port 443)
- Domain names resolving to your server's public IP
- Port 443 accessible from the internet (for ACME validation)
- Valid email address for Let's Encrypt account
- Writable `cache_dir` for certificate storage

## Troubleshooting

### Permission Denied on Ports 80/443
```bash
# Use sudo (quick solution)
sudo minipx --config /etc/minipx/config.json

# Or grant capabilities (Linux, persistent)
sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/minipx
```

### Certificate Acquisition Fails
1. Verify domain DNS points to your server
2. Ensure port 443 is open and accessible
3. Check email is valid: `minipx config email your@email.com`
4. Verify `cache_dir` is writable
5. Run with `--verbose` to see detailed error messages

### Backend Connection Refused
1. Verify backend service is running: `curl http://localhost:8080`
2. Check route configuration: `minipx routes show example.com`
3. Verify host/port are correct in config

### Configuration Issues
```bash
minipx config show-path  # Check which config is being used
minipx config show       # View current configuration
minipx --verbose         # Enable detailed logging
```

## Project Components

### [CLI (`cli/`)](cli/README.MD)
Complete command-line interface for running and managing minipx. Includes installation scripts, route management, and configuration tools.

### [Library (`minipx/`)](minipx/README.md)
Core library with modular components for configuration, proxy routing, SSL management, and IPC. Use in your own Rust projects.

### Web Interface (`web/`)
Optional web-based management interface for visualizing and managing proxy routes (coming soon).

### Tools (`tools/`)
Build and deployment utilities for creating release archives and automating builds.

## Architecture

Minipx follows a modular architecture:

```
┌─────────────┐
│  CLI / App  │  (Command line interface)
└──────┬──────┘
       │
       v
┌─────────────────────────────────────────┐
│         Minipx Core Library             │
│                                         │
│  ┌────────┐  ┌───────┐  ┌──────────┐  │
│  │ Config │  │ Proxy │  │   SSL    │  │
│  │        │  │       │  │  Server  │  │
│  └────────┘  └───────┘  └──────────┘  │
│                                         │
│  ┌────────┐  ┌───────┐                │
│  │  IPC   │  │ Utils │                │
│  └────────┘  └───────┘                │
└─────────────────────────────────────────┘
       │            │
       v            v
┌──────────┐  ┌──────────┐
│ Backend  │  │ Let's    │
│ Services │  │ Encrypt  │
└──────────┘  └──────────┘
```

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

### Development Setup

```bash
# Clone repository
git clone <repository-url>
cd minipx

# Build all components
cargo build

# Build specific component
cargo build -p minipx        # Library
cargo build -p minipx_cli    # CLI

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -p minipx_cli -- --verbose
```

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## Built With

- [Tokio](https://tokio.rs/) — Async runtime
- [Hyper](https://hyper.rs/) — HTTP implementation
- [Rustls](https://github.com/rustls/rustls) — TLS implementation
- [rustls-acme](https://github.com/FlorianUekermann/rustls-acme) — Let's Encrypt integration
- [Serde](https://serde.rs/) — Serialization framework
- [Clap](https://github.com/clap-rs/clap) — CLI argument parsing
- [Notify](https://github.com/notify-rs/notify) — File watching

## Related Projects

- [Caddy](https://caddyserver.com/) - Feature-rich web server with automatic HTTPS
- [Traefik](https://traefik.io/) - Modern reverse proxy and load balancer
- [Nginx](https://nginx.org/) - High-performance web server and proxy

## Support

- 📖 [CLI Documentation](cli/README.MD)
- 📖 [Library Documentation](minipx/README.md)
- 🎥 [Video Tutorial](https://youtu.be/2Uo7rTiK_h0)
- 🐛 [Issue Tracker](https://github.com/Drew-Chase/minipx/issues)

---

Made with ❤️ using Rust
