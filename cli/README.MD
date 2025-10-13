# Minipx CLI

Command-line interface for minipx, a fast and configurable reverse proxy with automatic SSL certificate management via Let's Encrypt.

## Features

- 🚀 **High Performance**: Built with Rust and Tokio
- 🔒 **Automatic SSL**: Let's Encrypt integration with ACME (TLS-ALPN-01)
- 🌐 **Multi-Domain**: Serve multiple domains with individual certificates
- 🔁 **Smart Redirects**: Optional HTTP→HTTPS redirects per route
- 🧩 **Hot Reload**: Live configuration updates with `--watch`
- 📊 **Logging**: Configurable logging levels
- 🛠️ **CLI Management**: Manage routes and configuration via command line

## Installation

### From Source

```bash
git clone <repository-url>
cd minipx
cargo build --release
```

The binary will be available at `target/release/minipx`.

### Linux (Easy Install)

Install using the provided script:

```bash
sudo bash -c "$(curl -sSL https://raw.githubusercontent.com/Drew-Chase/minipx/master/cli/install.sh)" < /dev/tty
```

### Linux Uninstall

```bash
sudo curl -sSL https://raw.githubusercontent.com/Drew-Chase/minipx/master/cli/install.sh | bash -s -- --uninstall
```

## Quick Start

### Running the Proxy

1. **Basic usage** (auto-detects config):
   ```bash
   minipx
   ```

2. **With custom config**:
   ```bash
   minipx --config ./minipx.json
   ```

3. **With hot-reload and verbose logging**:
   ```bash
   minipx --watch --verbose --config ./config/config.json
   ```

4. **Shortened form**:
   ```bash
   minipx -wvc /etc/minipx/config.json
   ```

## CLI Options

| Short | Long        | Description                                                 | Default         |
|-------|-------------|-------------------------------------------------------------|-----------------|
| `-h`  | `--help`    | Show help information                                       | -               |
| `-V`  | `--version` | Display version information                                 | -               |
| `-c`  | `--config`  | Path to the configuration file (overrides running instance) | `./minipx.json` |
| `-v`  | `--verbose` | Enable verbose logging (trace level)                        | `false`         |
| `-w`  | `--watch`   | Watch configuration file for changes (hot-reload)           | `false`         |

## Subcommands

### Routes Management

Manage proxy routes via the CLI.

#### List all routes
```bash
minipx routes list
```

#### Show a specific route
```bash
minipx routes show example.com
```

#### Add a new route
```bash
minipx routes add <domain> [OPTIONS]

# Example:
minipx routes add example.com --host 127.0.0.1 --port 8080 --path api --ssl --redirect
```

**Options:**
- `-j, --host <HOST>` - Backend host (default: localhost)
- `-p, --path <PATH>` - Backend path (e.g., /api/v1) (default: "")
- `-P, --port <PORT>` - Backend port (required, cannot be 80 or 443)
- `-s, --ssl` - Enable SSL (default: false)
- `-l, --listen-port <PORT>` - Custom listen port
- `-r, --redirect` - Redirect HTTP to HTTPS (default: false)

#### Update a route
```bash
minipx routes update <domain> [OPTIONS]

# Example:
minipx routes update example.com --port 9090 --no-redirect
```

**Options:**
- `-j, --host <HOST>` - Backend host
- `-p, --path <PATH>` - Backend path
- `-P, --port <PORT>` - Backend port
- `-s, --ssl` - Enable SSL
- `--no-ssl` - Disable SSL
- `-r, --redirect` - Enable HTTP→HTTPS redirect
- `--no-redirect` - Disable redirect

#### Remove a route
```bash
minipx routes remove <domain>

# Example:
minipx routes remove example.com
```

#### Add a subroute
```bash
minipx routes addsub <domain> <path> <port>

# Example:
minipx routes addsub example.com /maps/smp 8100
```

Subroutes allow path-based routing under a domain. The path prefix is stripped before proxying to the backend.

### Configuration Management

Manage the configuration file via the CLI.

#### Show current configuration
```bash
minipx config show
```

#### Set ACME email
```bash
minipx config email <email>

# Example:
minipx config email admin@example.com
```

#### Show configuration file path
```bash
minipx config show-path
```

## Configuration File

Minipx uses a JSON configuration file. See the [library documentation](../minipx/README.md) for detailed configuration format and options.

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
      "ssl_enable": true,
      "redirect_to_https": true
    },
    "app.mydomain.com": {
      "host": "192.168.1.100",
      "port": 8080,
      "ssl_enable": true,
      "redirect_to_https": true
    }
  }
}
```

## How It Works

### Server Behavior

- **HTTP Server**: Listens on port 80 (configurable per-route with `listen_port`)
- **HTTPS Server**: Listens on port 443, handles ACME challenges and TLS
- **Additional Listeners**: Spawned for routes with custom `listen_port` values
- **Smart Redirects**: HTTP→HTTPS redirects only occur if certificate is available

### Config Resolution Priority

1. Explicit `--config` / `-c` flag (highest priority)
2. Config path from running instance via IPC
3. `./minipx.json` (default)

### Inter-Process Communication (IPC)

When you run minipx, it starts a local IPC server that advertises its config path. This allows:

- CLI commands to discover the running instance's configuration
- Management of the running instance without specifying config path
- Single-instance coordination

**Security**: The IPC socket is local-only and not exposed over the network.

## CLI Examples

### Basic Operations

```bash
# Start with default config
minipx

# Start with custom config and verbose logging
minipx --verbose --config /etc/minipx/config.json

# Enable hot-reload
minipx --watch --verbose
```

### Route Management

```bash
# List all routes
minipx routes list

# Add a new HTTPS route with redirect
minipx routes add api.example.com \
  --host 127.0.0.1 \
  --port 8080 \
  --path /api/v1 \
  --ssl \
  --redirect

# Add a route on custom port
minipx routes add game.example.com \
  --host 192.168.1.50 \
  --port 7777 \
  --listen-port 25565

# Show specific route
minipx routes show api.example.com

# Update route settings
minipx routes update api.example.com --port 9090 --no-redirect

# Add a subroute (path-based routing)
minipx routes addsub example.com /maps/smp 8100

# Remove a route
minipx routes remove api.example.com
```

### Configuration

```bash
# Show current configuration (pretty-printed)
minipx config show

# Set ACME email for Let's Encrypt
minipx config email admin@mydomain.com

# Show the effective config path being used
minipx config show-path
```

## Video Tutorial

Watch a step-by-step tutorial on using minipx:

<a href="https://youtu.be/2Uo7rTiK_h0" target="_blank" rel="noopener noreferrer">
  <img src="https://img.youtube.com/vi/2Uo7rTiK_h0/maxresdefault.jpg" alt="Minipx tutorial on YouTube" width="640" />
</a>

## SSL / ACME (Let's Encrypt)

### Requirements

- Domains must resolve to your server's public IP
- Port 443 must be publicly accessible
- Valid email address for ACME registration

### Behavior

- **Automatic Certificates**: Requested for all domains with `ssl_enable: true`
- **Validation Method**: TLS-ALPN-01 (via port 443)
- **Certificate Cache**: Stored in `cache_dir` to avoid rate limits
- **Auto-Renewal**: Handled automatically by rustls-acme

### Troubleshooting SSL

1. Verify DNS records point to your server
2. Ensure port 443 is open and accessible
3. Check that `cache_dir` is writable
4. Confirm email is valid: `minipx config email your@email.com`
5. Check logs with `--verbose` for detailed error messages

## Logging

Control log verbosity:

```bash
# Info level (default)
minipx

# Trace level (verbose)
minipx --verbose

# Custom log level via environment variable
RUST_LOG=debug minipx
```

Log levels: `error`, `warn`, `info`, `debug`, `trace`

## Troubleshooting

### Permission Denied (Ports 80/443)

On Linux/macOS, binding to ports below 1024 requires elevated privileges:

```bash
# Option 1: Use sudo
sudo minipx --config /etc/minipx/config.json

# Option 2: Use capabilities (Linux)
sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/minipx
minipx --config /etc/minipx/config.json
```

### Port Already in Use

Check if another service is using the port:

```bash
# Linux
sudo netstat -tlnp | grep :80
sudo lsof -i :443

# Stop conflicting service
sudo systemctl stop apache2  # or nginx, etc.
```

### Backend Connection Refused

Verify the backend service is running:

```bash
# Check if service is listening
curl http://localhost:8080

# Check configuration
minipx routes show example.com
```

### Configuration Not Found

Check the config path:

```bash
# Show effective config path
minipx config show-path

# Specify explicit path
minipx --config /path/to/config.json
```

## Environment Variables

- `RUST_LOG` - Set logging level (e.g., `debug`, `trace`, `info`)
- Standard Rust environment variables for debugging

## Contributing

Contributions are welcome! Please submit issues and pull requests to the main repository.

## License

MIT License - See [LICENSE](../LICENSE) file for details.

## See Also

- [Minipx Library Documentation](../minipx/README.md) - For programmatic usage
- [Main Repository](../) - Project overview and architecture
