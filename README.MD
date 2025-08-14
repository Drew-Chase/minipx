# Minipx

A simple, fast, and configurable TCP/IP reverse proxy written in Rust with automatic SSL certificate management via Let's Encrypt.

## Features

- 🚀 **High Performance**: Built with Rust and Tokio for excellent performance and memory safety
- 🔒 **Automatic SSL**: Integrated Let's Encrypt support with automatic certificate provisioning and renewal
- 📝 **Configuration Management**: JSON-based configuration with hot-reload support
- 🔄 **Live Updates**: Watch configuration files for changes and apply them without restart
- 🌐 **Multi-Domain**: Support for multiple domains and routing rules
- 📊 **Logging**: Configurable logging levels for monitoring and debugging

## Quick Start

### Installation

Download the latest release from the [releases page](https://github.com/Drew-Chase/minipx/releases) or build from source:

```bash
git clone <repository-url>
cd minipx
cargo build --release
```

### Basic Usage

1. Create a configuration file (see [Configuration](#configuration) section):

```bash
minipx --config ./minipx.json
```

2. Run with configuration watching enabled:

```bash
minipx --watch --verbose --config ./config/config.json
```

> **Note**: The configuration directory will be created automatically if it doesn't exist.

## CLI Options

| Short | Long        | Description                          | Default         |
|-------|-------------|--------------------------------------|-----------------|
| `-h`  | `--help`    | Show help information                | -               |
| `-V`  | `--version` | Display version information          | -               |
| `-c`  | `--config`  | Path to the configuration file       | `./minipx.json` |
| `-v`  | `--verbose` | Enable verbose logging output        | `false`         |
| `-w`  | `--watch`   | Watch configuration file for changes | `false`         |

### Examples

```bash
# Basic usage with default config
minipx

# Use custom config with verbose logging
minipx --verbose --config /etc/minipx/config.json

# Enable hot-reload with configuration watching
minipx --watch --verbose --config ./config/config.json

# You can also concat the short options together
minipx -wvc /etc/minipx/config.json
```

## Configuration

Minipx uses a JSON configuration file to define proxy behavior, SSL settings, and routing rules.

### Global Settings

| Field       | Type   | Description                              | Default             |
|-------------|--------|------------------------------------------|---------------------|
| `email`     | string | Email for Let's Encrypt SSL certificates | `email@example.com` |
| `port`      | number | Port for the proxy server to listen on   | `80`                |
| `cache_dir` | string | Directory for SSL certificate cache      | `./cache`           |
| `routes`    | object | Domain-to-route mapping configuration    | `{}`                |

### Route Configuration

Each route in the `routes` object maps a domain to proxy settings:

| Field               | Type    | Description                                    | Default     |
|---------------------|---------|------------------------------------------------|-------------|
| `host`              | string  | Target hostname to proxy requests to           | `localhost` |
| `path`              | string  | Base path for proxied requests (no trailing /) | `""`        |
| `port`              | number  | Target port for the backend service            | `8080`      |
| `protocol`          | string  | Backend protocol (`http` or `https`)           | `http`      |
| `redirect_to_https` | boolean | Force HTTPS redirect for HTTP requests         | `false`     |

### Example Configuration

```json
{
  "email": "admin@mydomain.com",
  "port": 80,
  "cache_dir": "./ssl-cache",
  "routes": {
    "api.mydomain.com": {
      "host": "localhost",
      "path": "/api/v1",
      "port": 3000,
      "protocol": "http",
      "redirect_to_https": true
    },
    "app.mydomain.com": {
      "host": "192.168.1.100",
      "port": 8080,
      "protocol": "http",
      "redirect_to_https": true
    },
    "secure.mydomain.com": {
      "host": "backend-server",
      "port": 443,
      "protocol": "https",
      "redirect_to_https": false
    }
  }
}
```

## SSL Certificate Management

Minipx automatically handles SSL certificates using Let's Encrypt:

- **Automatic Provisioning**: Certificates are automatically requested when a new domain is accessed
- **Auto-Renewal**: Certificates are automatically renewed before expiration
- **Caching**: Certificates are cached in the specified `cache_dir` to avoid rate limits
- **ACME Challenge**: Uses HTTP-01 challenge type for domain validation

### Requirements for SSL

- Domain must resolve to the server running Minipx
- Port 80 must be accessible for ACME challenges
- Valid email address must be provided in configuration

## Logging

Minipx uses structured logging with different levels:

- **Error**: Critical errors and failures
- **Warn**: Warning messages and recoverable errors
- **Info**: General information about proxy operations
- **Debug**: Detailed debugging information (enabled with `--verbose`)
- **Trace**: Very detailed execution traces (enabled with `--verbose`)

Set the `RUST_LOG` environment variable for custom log levels:

```bash
RUST_LOG=debug minipx --config config.json
```

## Troubleshooting

### Common Issues

1. **Port 80 Permission Denied**
    - Run with sudo or use a port >= 1024
    - Ensure no other service is using port 80

2. **SSL Certificate Failures**
    - Verify domain DNS points to your server
    - Check that port 80 is accessible from the internet
    - Ensure email address is valid

3. **Backend Connection Refused**
    - Verify target service is running and accessible
    - Check host, port, and protocol settings in configuration

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
minipx --verbose --config config.json
```

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## License

This project is licensed under the [MIT License—](LICENSE)see the LICENSE file for details.

## Dependencies

Built with these excellent Rust crates:

- **tokio** - Async runtime
- **hyper** - HTTP implementation
- **rustls** - TLS implementation
- **rustls-acme** - Let's Encrypt integration
- **serde** - Serialization framework
- **clap** - Command line argument parsing