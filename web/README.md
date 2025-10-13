# Minipx Web Dashboard

A modern, feature-rich web dashboard for managing minipx proxy servers with real-time monitoring and resource visualization.

## Features

- **Modern UI**: Built with React 18, TypeScript, HeroUI, and TailwindCSS
- **Dark/Light Theme**: Fully themeable with automatic preference detection
- **Accessibility**: Colorblind-friendly theme options (Protanopia, Deuteranopia, Tritanopia)
- **Server Management**: Create, start, stop, restart, and delete proxy servers
- **Binary Upload**: Upload binaries or archives (ZIP, TAR) with automatic extraction
- **SSL/TLS Management**: Manage certificates with Let's Encrypt integration or custom uploads
- **Real-time Monitoring**: System resource tracking (CPU, Memory, Disk, Network)
- **Animations**: Smooth, bubbly animations using Framer Motion
- **Responsive Design**: Works on desktop, tablet, and mobile devices

## Tech Stack

### Frontend
- **React 18** - UI framework
- **TypeScript** - Type safety
- **HeroUI** - Modern React component library
- **TailwindCSS 3** - Utility-first CSS framework
- **Framer Motion** - Animation library
- **Vite** - Build tool and dev server
- **React Router** - Client-side routing

### Backend
- **Rust** - Backend language
- **Actix Web** - Web framework
- **SQLx** - Database toolkit
- **SQLite** - Database
- **Tokio** - Async runtime
- **Sysinfo** - System metrics

## Installation

### Prerequisites
- Node.js 18+ (for frontend)
- pnpm (package manager)
- Rust 1.70+ (for backend)
- Cargo

### Setup

1. **Install dependencies:**
   ```bash
   cd web
   pnpm install
   ```

2. **Build the Rust backend:**
   ```bash
   cargo build --release -p minipx_web
   ```

## Development

### Start Development Server

Run the frontend development server with hot reload:

```bash
cd web
pnpm run dev
```

The dashboard will be available at `http://localhost:3000`

In development mode, Vite automatically starts and provides HMR (Hot Module Replacement).

### Build Production Version

Build the optimized production bundle:

```bash
cd web
pnpm run build
```

This compiles TypeScript, bundles assets, and outputs to `target/wwwroot/`.

### Run Production Server

Start the Actix web server:

```bash
cd web
cargo run --bin minipx_web --release
```

The production server runs at `http://localhost:8080`

## Project Structure

```
web/
├── src/
│   ├── components/      # Reusable React components
│   │   └── Navigation.tsx
│   ├── pages/           # Page components
│   │   ├── Dashboard.tsx
│   │   ├── Servers.tsx
│   │   └── Certificates.tsx
│   ├── providers/       # React context providers
│   │   └── ThemeProvider.tsx
│   ├── types/           # TypeScript type definitions
│   │   └── index.ts
│   ├── utils/           # Utility functions
│   │   └── api.ts
│   ├── assets/          # Static assets
│   │   └── css/
│   └── main.tsx         # Application entry point
├── src-actix/           # Rust backend
│   ├── models.rs        # Database models
│   ├── db.rs            # Database connection
│   ├── server_endpoint.rs
│   ├── certificate_endpoint.rs
│   ├── metrics_endpoint.rs
│   └── lib.rs           # Main server setup
├── migrations/          # Database migrations
├── tailwind.config.js   # TailwindCSS configuration
├── vite.config.ts       # Vite configuration
└── package.json         # Node.js dependencies
```

## API Endpoints

### Servers
- `GET /api/servers` - List all servers
- `POST /api/servers` - Create new server
- `GET /api/servers/:id` - Get server details
- `PUT /api/servers/:id` - Update server
- `DELETE /api/servers/:id` - Delete server
- `POST /api/servers/:id/start` - Start server
- `POST /api/servers/:id/stop` - Stop server
- `POST /api/servers/:id/restart` - Restart server
- `POST /api/servers/upload` - Upload binary/archive

### Certificates
- `GET /api/certificates` - List all certificates
- `POST /api/certificates` - Create certificate
- `GET /api/certificates/:id` - Get certificate details
- `DELETE /api/certificates/:id` - Delete certificate
- `POST /api/certificates/upload` - Upload certificate files

### Metrics
- `GET /api/metrics/system` - Get system-wide stats
- `GET /api/metrics/server/:id` - Get server metrics
- `GET /api/metrics/server/:id/history` - Get historical metrics

## Theme System

The dashboard includes multiple theme options:

### Default Themes
- **Light** - Clean, bright interface
- **Dark** - Easy on the eyes for extended use

### Colorblind-Friendly Themes
- **Protanopia** (Red-blind)
- **Deuteranopia** (Green-blind)
- **Tritanopia** (Blue-blind)

Switch themes using the palette icon in the navigation bar.

## Creating a Server

1. Click **"Create Server"** button
2. Fill in the form:
   - **Server Name**: Display name (e.g., "My API")
   - **Domain**: Domain name (e.g., "api.example.com")
   - **Backend Host**: Target host (default: "localhost")
   - **Backend Port**: Target port (e.g., 8080)
   - **Path Prefix**: Optional path prefix (e.g., "/api/v1")
   - **Listen Port**: Optional custom listen port
   - **SSL/HTTPS**: Enable SSL certificate
   - **Redirect to HTTPS**: Auto-redirect HTTP to HTTPS
3. Optionally upload a binary or archive
4. Click **"Create Server"**

The server will be registered with minipx and added to the routing configuration.

## Managing Certificates

### Let's Encrypt (Automatic)
1. Click **"Add Certificate"**
2. Enter name and domain
3. Keep "Use Let's Encrypt" enabled
4. Click **"Add Certificate"**

The certificate will be automatically provisioned and renewed.

### Custom Certificate
1. Click **"Add Certificate"**
2. Enter name and domain
3. Disable "Use Let's Encrypt"
4. Upload certificate file (.pem)
5. Optionally upload private key (.pem)
6. Click **"Add Certificate"**

## Monitoring

The dashboard provides real-time monitoring of:

- **CPU Usage**: Processor utilization percentage
- **Memory Usage**: RAM consumption and total available
- **Disk Usage**: Storage space used and available
- **Network**: Incoming and outgoing data transfer

Metrics refresh every 5 seconds automatically.

## File Upload

The dashboard supports uploading:

### Supported Formats
- **Binaries**: Any executable file
- **Archives**: .zip, .tar, .gz, .tgz

When uploading an archive:
1. The file is uploaded to the server
2. Automatically extracted to the server directory
3. Ready to be executed by minipx

Maximum upload size: **512 MB**

## Troubleshooting

### Port Already in Use
If port 8080 is already in use, modify `src-actix/lib.rs`:

```rust
const PORT: u16 = 8080; // Change to desired port
```

### Database Connection Issues
The database file `minipx.db` is created automatically in the web directory. Ensure the directory is writable.

### CORS Errors
CORS is enabled by default for all origins in development. For production, update the CORS configuration in `src-actix/lib.rs`.

### Build Errors
Ensure all dependencies are installed:

```bash
pnpm install
cargo build
```

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Use pnpm for package management
2. Follow TypeScript best practices
3. Use functional components with hooks
4. Maintain responsive design principles
5. Test across different themes

## License

MIT License - See LICENSE file for details
