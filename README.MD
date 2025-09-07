# Minipx

A simple, fast, and configurable TCP/IP reverse proxy written in Rust with automatic SSL certificate management via Let's Encrypt.

## Features

- 🚀 High Performance: Built with Rust and Tokio for excellent performance and memory safety
- 🔒 Automatic SSL (HTTPS): Integrated Let's Encrypt support with automatic certificate provisioning and renewal via ACME (TLS-ALPN-01)
- 🌐 Multi-Domain: Serve multiple domains with individual certificates (no wildcard certs)
- 🔁 Smart Redirects: Optional per-route HTTP→HTTPS redirects that only occur when TLS is actually available for that host
- 🧩 Config Hot-Reload: JSON-based configuration with live updates and graceful restarts of the HTTPS server when needed
- 🧰 SSL Toggle: Global config flag to enable/disable the HTTPS server without changing routes
- 📊 Logging: Configurable logging levels for monitoring and debugging

## Quick Start

### Installation

Download the latest release from the releases page or build from source:

```bash
git clone <repository-url>
cd minipx
cargo build --release
```

### Linux (Easy)

Install using the provided script:

```bash
sudo bash -c "$(curl -sSL https://raw.githubusercontent.com/Drew-Chase/minipx/master/install.sh)" < /dev/tty
```

### Linux Uninstall (Easy)

```bash
sudo curl -sSL https://raw.githubusercontent.com/Drew-Chase/minipx/master/install.sh | bash -s -- --uninstall
```

### Basic Usage

1) Create a configuration file (see Configuration below), then run:

```bash
minipx --config ./minipx.json
```

2) Run with configuration watching enabled (hot-reload):

```bash
minipx --watch --verbose --config ./config/config.json
```

> Note: The configuration directory will be created automatically if it doesn't exist.

## Video Tutorial

Watch a step-by-step tutorial on how to use minipx:

<a href="https://youtu.be/2Uo7rTiK_h0" target="_blank" rel="noopener noreferrer">
  <img src="https://img.youtube.com/vi/2Uo7rTiK_h0/maxresdefault.jpg" alt="Minipx tutorial on YouTube" width="640" />
</a>

## How It Runs

- HTTP server listens on the configured port (default 80) on 0.0.0.0.
- HTTPS server listens on port 443 on [::] (all IPv6/IPv4 via dual-stack), automatically handling ACME TLS-ALPN-01 challenges and serving TLS.
- Additional HTTP listeners are spawned on startup for any routes with a listen_port configured (excluding 80 and 443).
- When a route has redirect_to_https = true, HTTP requests are redirected to HTTPS only if a certificate can be served for that host; otherwise, the request is served over HTTP, and a warning is logged.
- Config path resolution priority: explicit CLI --config/-c > config path from a running minipx instance via local IPC > ./minipx.json (default).
- On startup, minipx starts a lightweight local IPC server that advertises its effective config path for tooling and subsequent CLI interactions.

## CLI Options

| Short | Long        | Description                                                 | Default         |
|-------|-------------|-------------------------------------------------------------|-----------------|
| `-h`  | `--help`    | Show help information                                       | -               |
| `-V`  | `--version` | Display version information                                 | -               |
| `-c`  | `--config`  | Path to the configuration file (overrides running instance) | `./minipx.json` |
| `-v`  | `--verbose` | Enable verbose logging                                      | `false`         |
| `-w`  | `--watch`   | Watch configuration file for changes (hot-reload)           | `false`         |

### Subcommands

- **routes** - Manage proxy routes
    - **add** - Add a new proxy route
        - **required:** `domain` - The domain name for the route (e.g., example.com)
        - **flags:**
            - `-j, --host` - The redirect host (default: localhost)
            - `-p, --path` - Path to route to (e.g. /api/v1) (default: "")
            - `-P, --port` - Port to route to, cannot be 80 or 443, and must be between 1 and 65535
            - `-s, --ssl` - Enable SSL (default: false)
            - `-l, --listen-port` - Port to listen on for incoming requests, defaults to 80 for HTTP and 443 for HTTPS
            - `-r, --redirect` - Redirect HTTP to HTTPS (default: false)
    - **remove** - Remove a proxy route
        - **required:** `host` - The host/domain to remove
    - **list** - List all proxy routes
    - **show** - Show a single proxy route
        - **required:** `host` - The host/domain to show
    - **update** - Partially update a proxy route by domain key
        - **required:** `domain` - Domain of the route to update (the route key, e.g., example.com)
        - **flags:**
            - `-j, --host` - Backend host (e.g. 127.0.0.1)
            - `-p, --path` - Backend path (e.g., web or api/v1) — do not start with '/'
            - `-P, --port` - Backend port (1..=65535, not 80/443)
            - `-s, --ssl` - Enable SSL for this route (frontend terminates TLS)
            - `--no-ssl` - Disable SSL for this route
            - `-r, --redirect` - Redirect HTTP to HTTPS for this route
            - `--no-redirect` - Disable HTTP to HTTPS redirect
- **config** - Manage the configuration file
    - **show** - Show the current configuration
    - **email** - Set the email address to use for SSL certificates
        - **required:** `email` - Email address for ACME/Let's Encrypt
    - **show-path** - Show the path to the configuration file

### Examples

```bash
# Basic usage with default/auto-detected config (via IPC if another instance is running)
minipx

# Use custom config with verbose logging (overrides any running instance)
minipx --verbose --config /etc/minipx/config.json

# Enable hot-reload with configuration watching
minipx --watch --verbose --config ./config/config.json

# You can also concat the short options together
minipx -wvc /etc/minipx/config.json

# List all routes
minipx routes list

# Add a new route
minipx routes add example.com --host 127.0.0.1 --port 8080 --path api --redirect

# Show a route by host lookup
minipx routes show example.com

# Update only specific fields on an existing route (by domain key)
minipx routes update example.com --port 9090 --no-redirect

# Remove a route
minipx routes remove example.com

# Show current config (pretty-printed)
minipx config show

# Set ACME email (will be saved to config)
minipx config email admin@mydomain.com

# Show the effective config path in use
minipx config show-path
```

## Inter-Process Communication (IPC)

Minipx runs a small local IPC service to improve UX for tools and CLI:

- Purpose: Advertise the effective configuration file path of the running instance.
- Transport: Cross-platform local socket / named pipe.
- Behavior:
    - When you run minipx, it starts an IPC listener in the background that serves its config path to any local client.
    - CLI runs (without -c/--config) will first attempt to retrieve the config path from a running instance; if none is found, they fall back to ./minipx.json.
    - Supplying -c/--config always takes precedence and bypasses IPC discovery.
- Single-instance friendly: If another instance is already listening, new instances won’t replace the IPC listener and will log that binding failed.
- Security: The socket is local-only and not exposed over the network.

## Configuration

Minipx uses a JSON configuration file to define proxy behavior, SSL settings, and routing rules.

### Global Settings

| Field       | Type   | Description                                          | Default     |
|-------------|--------|------------------------------------------------------|-------------|
| `email`     | string | Email for Let's Encrypt account (required for HTTPS) | `""`        |
| `cache_dir` | string | Directory for ACME account/certificate cache         | `"./cache"` |
| `routes`    | object | Domain-to-route mapping configuration                | `{}`        |

### Route Configuration

Each entry under `routes` maps a domain (e.g., "api.example.com") to proxy settings.

| Field                      | Type    | Description                                         | Default       |
|----------------------------|---------|-----------------------------------------------------|---------------|
| `host`                     | string  | Target hostname to proxy requests to                | `"localhost"` |
| `path`                     | string  | Base path for proxied requests (no trailing /)      | `""`          |
| `port`                     | number  | Target port for the backend service                 | `0`           |
| `ssl_enable`               | boolean | Enable SSL for this route (frontend terminates TLS) | `false`       |
| `redirect_to_https`        | boolean | Force HTTPS redirect for HTTP requests              | `false`       |
| `listen_port` _(optional)_ | number  | Additional port to bind for this route              | `null`        |

Notes:

- Wildcard route keys (like "*.example.com") are supported for routing lookups, but wildcard certificates are NOT requested. Only exact domains are used for ACME.

### Custom Listener Ports (per-route)

You can bind Minipx to additional inbound ports on a per-route basis using the `listen_port` field. On startup, Minipx spawns an extra HTTP listener for each unique `listen_port` specified (excluding 80 and 443). Requests received on those ports are routed by Host header just like on port 80.

Examples (route entries):

```json
{
  "routes": {
    "api.example.com": {
      "host": "127.0.0.1",
      "path": "/api/v1",
      "port": 1234,
      "ssl_enable": false,
      "redirect_to_https": false
    },
    "mc.example.com": {
      "host": "127.0.0.1",
      "path": "",
      "port": 7894,
      "listen_port": 25565,
      "ssl_enable": false,
      "redirect_to_https": false
    }
  }
}
```

Routes explanation:

- `api.example.com` redirects traffic from api.example.com:[80/443]/api/v1 -> 127.0.0.1:1234
- `mc.example.com` redirects traffic from mc.example.com:25565 -> 127.0.0.1:7894

Notes:

- Ports 80 (HTTP) and 443 (HTTPS) are reserved for the default listeners and should not be set as `listen_port`.
- Changing `listen_port` currently requires a restart to bind/unbind that port; other routing changes apply live.

### Example Configuration

```json
{
  "email": "admin@mydomain.com",
  "cache_dir": "./ssl-cache",
  "routes": {
    "api.mydomain.com": {
      "host": "localhost",
      "path": "/api/v1",
      "port": 3000,
      "ssl_enable": false,
      "redirect_to_https": true
    },
    "app.mydomain.com": {
      "host": "192.168.1.100",
      "port": 8080,
      "ssl_enable": false,
      "redirect_to_https": true
    },
    "secure.mydomain.com": {
      "host": "backend-server",
      "port": 443,
      "ssl_enable": true,
      "redirect_to_https": false
    }
  }
}
```

## SSL / ACME (Let's Encrypt)

Minipx uses rustls-acme with the Let's Encrypt production directory.

- Validation: TLS-ALPN-01 (served on port 443). No HTTP-01 is used.
- Certificate caching: Stored in `cache_dir` to avoid rate limits.
- Multi-domain: Certificates are requested for all valid, exact domains present as keys in `routes`.
- Wildcards: Not supported for certificates. Wildcard route keys are ignored for ACME.

Requirements:

- Domains must resolve to this server.
- Port 443 must be publicly accessible for TLS-ALPN-01 validation and HTTPS traffic.
- A valid email must be provided for the ACME account. If the email is invalid, the HTTPS server won’t start and will wait for a valid config.

Behavior with invalid config:

- If no routes have `ssl_enable` set to true: HTTPS server does not start (waits for SSL-enabled routes via hot-reload).
- If `email` is invalid: HTTPS server waits until a valid email is provided.
- If some domains are invalid: HTTPS starts for valid domains only and logs the skipped ones.
- HTTP→HTTPS redirects occur only when a certificate can be served for that host; otherwise traffic is served over HTTP and a warning is logged.

Hot-reload behavior:

- Changes to route `ssl_enable` settings, `email`, `cache_dir`, or the set of valid domains trigger a graceful HTTPS server restart.

## Logging

Minipx uses structured logging with different levels:

- Error: Critical errors and failures
- Warn: Warning messages and recoverable errors
- Info: General information about proxy operations
- Debug/Trace: More detailed logs (enabled with `--verbose`)

Set the `RUST_LOG` environment variable for custom log levels:

```bash
RUST_LOG=debug minipx --config config.json
```

## Troubleshooting

Common issues:

1) Binding to privileged ports (80/443)

- On Linux/macOS, run with sudo or use capabilities (e.g., setcap) to bind <1024.
- Ensure no other service is using the port.

2) Certificate acquisition failures

- Verify DNS records resolve to this server.
- Ensure port 443 is reachable from the internet.
- Check that `cache_dir` is writable and persistent.
- Confirm `email` is valid.

3) Backend connection refused

- Verify target service is running and reachable from Minipx.
- Check `host` and `port` in your route configuration.

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## Dependencies

Built with these excellent Rust crates:

- tokio — Async runtime
- hyper — HTTP implementation
- rustls — TLS implementation
- rustls-acme — Let's Encrypt integration
- serde — Serialization framework
- clap — Command line argument parsing