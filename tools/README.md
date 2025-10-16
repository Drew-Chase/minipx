# Cross-Compilation Tools

This directory contains the cross-compilation tool for building minipx on different platforms using Docker.

## Prerequisites

1. **Docker** - Must be installed and running
   - Windows: [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop/)
   - macOS: [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop/)
   - Linux: Install via your package manager

2. **cross** - Cross-compilation tool
   ```bash
   cargo install cross --git https://github.com/cross-rs/cross
   ```

3. **Rust targets** - Will be installed automatically by the tool if missing

## Quick Start

The cross-build tool is available via a cargo alias. From the project root, run:

```bash
# Build all variants for ARM64 Linux (default)
cargo build-cross

# Build for specific target
cargo build-cross --target aarch64-unknown-linux-gnu
cargo build-cross --target x86_64-unknown-linux-gnu

# Build for all platforms (Linux, macOS, Windows - x64 and ARM64)
cargo build-cross --target all

# Build specific variant
cargo build-cross --variant cli
cargo build-cross --variant cli-webui
cargo build-cross --variant web
cargo build-cross --variant all

# Clean build
cargo build-cross --clean

# Create zip archives in target/dist
cargo build-cross --archive

# Build all variants for all platforms with archives
cargo build-cross --target all --archive

# Combine options
cargo build-cross --target x86_64-unknown-linux-gnu --variant cli --clean --archive
```

## Usage

### Basic Commands

```bash
# Default: Build all variants for ARM64 Linux
cargo build-cross

# Help message
cargo build-cross --help
```

### Options

- `-t, --target <TARGET>` - Target platform or `all` for all platforms (default: `aarch64-unknown-linux-gnu`)
  - `x86_64-unknown-linux-gnu` - Linux x64
  - `aarch64-unknown-linux-gnu` - Linux ARM64
  - `x86_64-apple-darwin` - macOS Intel
  - `aarch64-apple-darwin` - macOS Apple Silicon
  - `x86_64-pc-windows-msvc` - Windows x64
  - `all` - All platforms above
- `-v, --variant <VARIANT>` - Build variant: `cli`, `cli-webui`, `web`, or `all` (default: `all`)
- `-c, --clean` - Clean build artifacts before building
- `-a, --archive` - Create zip archives in target/dist after building
- `--verbose` - Verbose output

### Examples

```bash
# Build only CLI for ARM64 Linux
cargo build-cross --variant cli

# Build all variants for x86-64 Linux
cargo build-cross --target x86_64-unknown-linux-gnu

# Build for all platforms (Linux, macOS, Windows)
cargo build-cross --target all

# Build CLI only for all platforms
cargo build-cross --target all --variant cli

# Clean build of web variant for ARM64
cargo build-cross --variant web --clean

# Build CLI with WebUI for ARM64
cargo build-cross --variant cli-webui

# Build and create archives
cargo build-cross --archive

# Build all variants for all platforms with archives (full distribution build)
cargo build-cross --target all --archive

# Build specific variant and create archive
cargo build-cross --variant cli --archive

# Complete build with clean and archive
cargo build-cross --target x86_64-unknown-linux-gnu --clean --archive
```

## Supported Targets

- **x86_64-unknown-linux-gnu** - Linux x86-64
- **aarch64-unknown-linux-gnu** - Linux ARM64 (e.g., Raspberry Pi, AWS Graviton)
- **x86_64-apple-darwin** - macOS Intel
- **aarch64-apple-darwin** - macOS Apple Silicon (M1/M2/M3)
- **x86_64-pc-windows-msvc** - Windows x86-64
- **all** - Build for all platforms above

### Cross-Compilation Support

The `cross` tool uses Docker for cross-compilation. Support varies by platform:

| Target Platform | Works from Windows | Works from macOS | Works from Linux |
|----------------|-------------------|------------------|------------------|
| Linux x64/ARM64 | ✅ Yes | ✅ Yes | ✅ Yes |
| macOS x64/ARM64 | ❌ No* | ✅ Yes | ❌ No* |
| Windows x64 | ✅ Native | ❌ No* | ❌ No* |

**\* No Docker images available** - `cross` will fall back to the host compiler, which typically fails unless you have the target toolchain installed natively.

**Recommendation**:
- Build Linux targets from any platform (Docker-based)
- Build macOS targets on macOS (native)
- Build Windows targets on Windows (native)

If a build fails for an unsupported target, check the clickable **[open log]** link in the error message for details.

## Build Variants

1. **CLI only** (`cli`) - Pure reverse proxy CLI without web interface
   - Package: `minipx_cli`
   - Features: None
   - Output: `minipx`

2. **CLI with WebUI** (`cli-webui`) - CLI with embedded web management interface
   - Package: `minipx_cli`
   - Features: `webui`
   - Output: `minipx` (with web UI embedded)

3. **Web only** (`web`) - Standalone web management server
   - Package: `minipx_web`
   - Features: None
   - Output: `minipx_web`

## Output Location

### Built Binaries

Built binaries are located in:
```
target/<target-triple>/release/
```

For example:
```
target/aarch64-unknown-linux-gnu/release/minipx
target/aarch64-unknown-linux-gnu/release/minipx_web
```

### Archives

When using the `--archive` flag, zip archives are created in:
```
target/dist/
```

Archive naming convention:
```
minipx-{variant}-{os}-{arch}.zip
```

For example:
```
target/dist/minipx-cli-linux-arm64.zip
target/dist/minipx-cli-webui-linux-arm64.zip
target/dist/minipx-web-linux-arm64.zip
target/dist/minipx-cli-linux-x64.zip
target/dist/minipx-cli-macos-x64.zip
target/dist/minipx-cli-macos-arm64.zip
target/dist/minipx-cli-windows-x64.zip
```

When using `--target all --archive`, all 15 archives will be created (3 variants × 5 platforms).

Supported OS names: `linux`, `windows`, `macos`
Supported architectures: `x64`, `arm64`, `x86`, `armv7`

### Build Logs

Build output is automatically logged to files for debugging purposes:
```
target/logs/
```

Log file naming convention:
```
minipx-{variant}-{target}.log
```

For example:
```
target/logs/minipx-cli-x86_64-unknown-linux-gnu.log
target/logs/minipx-cli-webui-aarch64-unknown-linux-gnu.log
target/logs/minipx-web-x86_64-pc-windows-msvc.log
```

**Features:**
- All build output (stdout and stderr) is logged for each target-variant combination
- Logs are created before builds start and updated in real-time
- If a build fails, the error message includes a clickable **[open log]** link using OSC 8 hyperlinks
- Log files persist across builds for debugging historical issues
- Logs are stored with absolute paths for easy access from any terminal

**Viewing Failed Build Logs:**

When a build fails, the error message will show:
```
✗ x86_64-unknown-linux-gnu (cli) - Build failed - [open log]
```

The **`[open log]`** text is clickable in modern terminal emulators (Windows Terminal, iTerm2, Alacrity, VS Code terminal, JetBrains terminal, etc.). Click it to open the log file in your default text editor.

Alternatively, you can manually open the log:
```bash
# On Windows
notepad target\logs\minipx-cli-x86_64-unknown-linux-gnu.log

# On macOS/Linux
cat target/logs/minipx-cli-x86_64-unknown-linux-gnu.log
```

## How It Works

The `cargo build-cross` alias runs the `cross-build-tool` binary, which:

1. Checks if Docker is running
2. Verifies cross is installed
3. Ensures the target is added to rustup
4. Creates log directory (`target/logs`)
5. Runs cross builds in parallel with output logged to files
6. Shows progress with spinners, checkmarks (✓) for success, X (✗) for failures
7. (Optional) Creates zip archives in target/dist
8. Provides colored output and clickable log links for failed builds
9. **Handles Ctrl+C gracefully** - Stops all running Docker containers when cancelled

### Cancellation Handling

If you press **Ctrl+C** during a build:
1. The tool detects the signal immediately
2. All running builds are cancelled
3. Docker containers started by `cross` are automatically stopped and removed
4. The tool exits cleanly with proper cleanup

This prevents dangling Docker containers when you interrupt a build.

The alias is defined in `.cargo/config.toml`:
```toml
[alias]
build-cross = "run --package cross-build-tool --release --"
```

## Configuration

### Cross.toml

The `Cross.toml` file in the project root configures cross-compilation settings:
- Docker images for each target platform
- Build environment variables
- Custom build scripts

### Vendored OpenSSL

The project uses vendored OpenSSL compilation to avoid system library dependencies:
```toml
openssl = { version = "0.10", features = ["vendored"] }
```

This allows cross-compilation to work seamlessly without needing to install ARM64 OpenSSL libraries on the host system.

## Troubleshooting

### Docker not running
**Error**: `Docker is not running`

**Solution**: Start Docker Desktop (Windows/Mac) or Docker daemon (Linux)

```bash
# Windows/Mac: Start Docker Desktop application
# Linux:
sudo systemctl start docker
```

### Cross not found
**Error**: `cross is not installed`

**Solution**:
```bash
cargo install cross --git https://github.com/cross-rs/cross
```

### Image pull failures
**Error**: `Unable to find image 'ghcr.io/cross-rs/...'`

**Solution**: This is normal on first run. Cross will automatically download the Docker image. Ensure you have:
- Internet connectivity
- Sufficient disk space (images are ~1-2 GB)

### Build failures with OpenSSL
If builds fail with OpenSSL-related errors, ensure:
1. The `Cross.toml` file exists in the project root
2. The `openssl` dependency with `vendored` feature is in `minipx/Cargo.toml`
3. Docker has sufficient memory (increase in Docker Desktop settings if needed)

### Permission denied (Linux)
If you get permission errors on Linux:
```bash
# Add your user to the docker group
sudo usermod -aG docker $USER
# Log out and back in for changes to take effect
```

## Manual Cross-Compilation

You can also use `cross` directly without the tool:

```bash
# CLI only
cross build --release --target aarch64-unknown-linux-gnu -p minipx_cli

# CLI with WebUI
cross build --release --target aarch64-unknown-linux-gnu -p minipx_cli --features webui

# Web only
cross build --release --target aarch64-unknown-linux-gnu -p minipx_web
```

## Development

### Building the Tool

The cross-build tool itself is a Rust binary:

```bash
# Build the tool
cargo build -p cross-build-tool

# Run directly
cargo run -p cross-build-tool -- --help
```

### Source Code

The tool source is located at:
- `tools/cross-build-tool/src/main.rs` - Main implementation
- `tools/cross-build-tool/Cargo.toml` - Dependencies

## CI/CD Integration

The GitHub Actions release workflow (`.github/workflows/release.yml`) uses cross-compilation for:
- Linux ARM64 builds
- All three build variants (CLI, CLI+WebUI, Web)

The workflow automatically:
1. Installs cross
2. Adds required targets
3. Builds all variants
4. Creates platform-specific archives
5. Uploads artifacts to GitHub releases

## Additional Resources

- [cross documentation](https://github.com/cross-rs/cross)
- [Rust cross-compilation guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Docker documentation](https://docs.docker.com/)
- [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
