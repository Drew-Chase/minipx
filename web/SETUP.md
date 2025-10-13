# Minipx Dashboard Setup Guide

## Quick Start

This guide will help you set up and run the minipx web dashboard.

### Prerequisites

Before starting, ensure you have:

- **Node.js 18+**: [Download here](https://nodejs.org/)
- **pnpm**: Install with `npm install -g pnpm`
- **Rust 1.70+**: [Install from rustup.rs](https://rustup.rs/)
- **Cargo**: Comes with Rust installation

### Installation Steps

1. **Navigate to web directory:**
   ```bash
   cd web
   ```

2. **Install frontend dependencies:**
   ```bash
   pnpm install
   ```

3. **Start development server:**
   ```bash
   pnpm run dev
   ```

   The dashboard will be available at: http://localhost:3000

4. **In a new terminal, start the backend:**
   ```bash
   cargo run --bin minipx_web
   ```

   The API server will run at: http://localhost:8080

### Production Build

To build for production:

```bash
# Build frontend
pnpm run build-frontend

# Build backend
cargo build --release -p minipx_web

# Run production server
cargo run --bin minipx_web --release
```

## Project Features

### ✨ User Interface
- **Modern Design**: Clean, intuitive interface built with HeroUI components
- **Smooth Animations**: Powered by Framer Motion for delightful interactions
- **Responsive Layout**: Works seamlessly on desktop, tablet, and mobile
- **Theme Support**: Light and dark modes with smooth transitions

### ♿ Accessibility
- **Colorblind Modes**: Support for Protanopia, Deuteranopia, and Tritanopia
- **Keyboard Navigation**: Full keyboard support throughout the dashboard
- **Screen Reader Friendly**: Semantic HTML and ARIA labels
- **High Contrast**: Optimized color contrasts for readability

### 🖥️ Server Management
- **Create Servers**: Configure proxy routes with full minipx CLI options
- **Binary Upload**: Upload executables or archives (ZIP, TAR, GZ)
- **Auto-Extraction**: Archives are automatically extracted
- **Server Controls**: Start, stop, restart servers with one click
- **Real-time Status**: Live server status updates

### 🔐 SSL/TLS Management
- **Let's Encrypt**: Automatic certificate provisioning and renewal
- **Custom Certificates**: Upload your own cert and key files
- **Certificate Tracking**: Monitor expiry dates and certificate details

### 📊 Resource Monitoring
- **CPU Usage**: Real-time processor utilization
- **Memory Stats**: RAM usage with total and used metrics
- **Disk Space**: Storage utilization and available space
- **Network Traffic**: Incoming and outgoing data transfer
- **Auto-Refresh**: Metrics update every 5 seconds

## Color Scheme

The dashboard uses a modern, professional color palette:

### Light Theme
- **Primary**: Indigo (#6366f1) - Actions and highlights
- **Secondary**: Emerald (#10b981) - Success states
- **Background**: Slate (#f8fafc) - Clean, bright base
- **Warning**: Amber (#f59e0b) - Caution states
- **Danger**: Red (#ef4444) - Error states

### Dark Theme
- **Primary**: Light Indigo (#818cf8) - Softer on the eyes
- **Secondary**: Light Emerald (#34d399) - Vibrant success
- **Background**: Dark Slate (#0f172a) - Deep, comfortable base
- **Warning**: Light Amber (#fbbf24) - Visible warnings
- **Danger**: Light Red (#f87171) - Clear error indication

### Colorblind Themes
Each colorblind mode uses scientifically-selected color palettes to ensure maximum accessibility.

## Architecture Overview

### Frontend Stack
```
React 18 + TypeScript
├── HeroUI (Components)
├── TailwindCSS (Styling)
├── Framer Motion (Animations)
├── React Router (Navigation)
└── Vite (Build Tool)
```

### Backend Stack
```
Actix Web + SQLx
├── SQLite (Database)
├── Tokio (Async Runtime)
├── Sysinfo (System Metrics)
└── Minipx Library (Proxy Integration)
```

### Data Flow
```
User → React Components → API Calls → Actix Endpoints → SQLite/Minipx → Response
                                                              ↓
                                          System Metrics ← Sysinfo
```

## API Integration

All API calls are handled through `src/utils/api.ts`:

```typescript
import { serverAPI, certificateAPI, metricsAPI } from '../utils/api';

// Example: Create a server
await serverAPI.create({
  name: "My App",
  domain: "app.example.com",
  port: 8080,
  ssl_enabled: true
});
```

## Database Schema

The dashboard uses SQLite with the following tables:

- **servers**: Proxy server configurations
- **certificates**: SSL/TLS certificates
- **resource_metrics**: Historical resource usage data
- **server_certificates**: Many-to-many relationship

Migrations are automatically applied on startup.

## Development Tips

### Hot Reload
The dev server supports hot module replacement (HMR). Changes to React components are reflected instantly without page refresh.

### Type Safety
TypeScript provides full type checking. Run `tsc` to check for type errors before building:

```bash
pnpm exec tsc --noEmit
```

### Linting
ESLint is configured for code quality:

```bash
pnpm run lint
```

### Adding New Components
1. Create component in `src/components/`
2. Export from component file
3. Import where needed
4. Use HeroUI components for consistency

### Adding New Pages
1. Create page in `src/pages/`
2. Add route in `src/main.tsx`
3. Add navigation link in `src/components/Navigation.tsx`

## Troubleshooting

### "pnpm: command not found"
Install pnpm globally:
```bash
npm install -g pnpm
```

### Port 3000 or 8080 Already in Use
Change the ports in:
- `vite.config.ts` (frontend)
- `src-actix/lib.rs` (backend)

### Database Locked
Ensure no other instance of minipx_web is running:
```bash
pkill minipx_web
```

### Build Errors
Clean and rebuild:
```bash
# Frontend
rm -rf node_modules pnpm-lock.yaml
pnpm install

# Backend
cargo clean
cargo build
```

### CORS Issues
In development, CORS is wide open. For production, update the CORS middleware in `src-actix/lib.rs`.

## Next Steps

After setup, you can:

1. **Create Your First Server**: Click "Create Server" and configure a proxy route
2. **Upload an Application**: Use the file upload to deploy a binary or archive
3. **Monitor Resources**: Watch CPU, memory, disk, and network usage in real-time
4. **Manage Certificates**: Set up SSL with Let's Encrypt or upload custom certs
5. **Customize Themes**: Try different colorblind modes or toggle dark/light theme

## Support

For issues or questions:
- Check the main [README](README.md)
- Review the [CLAUDE.md](../CLAUDE.md) for architecture details
- Open an issue on GitHub

Happy proxying! 🚀
