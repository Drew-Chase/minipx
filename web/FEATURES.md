# Minipx Dashboard - Feature Documentation

## Complete Feature List

### 🎨 User Interface & Design

#### Modern, Bubbly Animations
- **Scale-in animations** on card hover
- **Fade-in effects** for page transitions
- **Slide-up/down** for modals and dropdowns
- **Bounce-subtle** for interactive elements
- **Pulse effects** on progress bars
- All animations powered by Framer Motion and Tailwind CSS

#### Responsive Design
- **Mobile-first** approach
- **Breakpoints**: sm (640px), md (768px), lg (1024px)
- **Adaptive layouts** that reorganize on smaller screens
- **Touch-friendly** buttons and interactions

#### Theme System
- **Light Mode**: Bright, professional interface
- **Dark Mode**: Easy on the eyes for extended sessions
- **Smooth transitions** between themes
- **LocalStorage persistence** - remembers your choice

#### Colorblind Accessibility
- **Protanopia Mode**: Red-blind friendly (Blue/Yellow palette)
- **Deuteranopia Mode**: Green-blind friendly (Purple/Orange palette)
- **Tritanopia Mode**: Blue-blind friendly (Pink/Cyan palette)
- **Scientific color selection** for maximum clarity
- **Persistent preferences** across sessions

### 🖥️ Server Management

#### Create Servers
- **Server Name**: Friendly display name
- **Domain Configuration**: Support for any domain or subdomain
- **Backend Host**: Target host (default: localhost)
- **Backend Port**: Port of the proxied application
- **Path Prefix**: Optional path like `/api/v1`
- **Listen Port**: Custom frontend port (optional)
- **SSL/HTTPS Toggle**: Enable automatic SSL
- **HTTP→HTTPS Redirect**: Automatic redirects
- **Integration**: Directly registers with minipx library

#### Server Cards
- **Status Indicators**: Running, Stopped, Error, Restarting
- **Visual Status**: Color-coded badges and icons
- **Quick Actions**: Start, Stop, Restart buttons
- **Configuration Display**: Shows host:port, path, SSL status
- **Feature Badges**: SSL, HTTPS Redirect, Custom Port
- **Hover Effects**: Scale-up animation on hover
- **Delete Confirmation**: Safety prompt before deletion

#### Binary/Archive Upload
- **Drag & Drop**: Easy file selection
- **Multiple Formats**:
  - Executables (any binary)
  - ZIP archives
  - TAR archives
  - GZ/TGZ compressed archives
- **Auto-Extraction**: Archives automatically unzipped/untarred
- **Large File Support**: Up to 512 MB
- **Progress Feedback**: Visual confirmation of uploads
- **Server Directory**: Each server gets isolated directory

#### Server Controls
- **Start**: Launch a stopped server
- **Stop**: Gracefully stop a running server
- **Restart**: Stop and start in one action
- **Delete**: Remove server and cleanup files
- **View Details**: Full configuration modal
- **Real-time Updates**: Status changes reflected immediately

### 🔐 Certificate Management

#### Let's Encrypt Integration
- **Automatic Provisioning**: Just enter domain
- **Auto-Renewal**: Handled by minipx
- **TLS-ALPN-01**: Secure validation method
- **Multi-Domain**: Separate cert per domain
- **Expiry Tracking**: Monitor certificate expiration

#### Custom Certificates
- **Manual Upload**: Bring your own certificates
- **Certificate File**: Upload .pem, .crt, .cer
- **Private Key**: Optional key file upload
- **Secure Storage**: Files stored in isolated directories
- **Domain Validation**: Ensure cert matches domain

#### Certificate Cards
- **Type Indicators**: Let's Encrypt vs Custom badges
- **Domain Display**: Primary domain name
- **Expiry Dates**: When certificate expires
- **Creation Dates**: When added to dashboard
- **Quick Delete**: Remove certificates easily
- **Color-Coded**: Different colors for different types

### 📊 Resource Monitoring

#### System-Wide Metrics
- **CPU Usage**: Real-time processor utilization
  - Percentage display
  - Visual progress bar
  - Color-coded (Green: <60%, Yellow: 60-80%, Red: >80%)
  - Status chips (Low/Medium/High)

- **Memory Usage**: RAM statistics
  - Percentage and absolute values
  - Used vs Total display
  - Human-readable format (KB, MB, GB)
  - Progress bar with thresholds
  - Color-coded status

- **Disk Usage**: Storage metrics
  - Percentage and absolute values
  - Used vs Total storage
  - Multi-disk aggregation
  - Threshold-based coloring
  - Visual progress indicator

- **Network Traffic**: Data transfer
  - Incoming (Download) traffic
  - Outgoing (Upload) traffic
  - Human-readable format
  - Real-time updates

#### Auto-Refresh
- **5-Second Intervals**: Metrics update automatically
- **No Page Reload**: Seamless updates via API
- **Efficient Polling**: Optimized to minimize load
- **Background Updates**: Works even when inactive

#### Quick Stats Dashboard
- **Total Servers**: Count of all servers
- **Running Servers**: Active server count
- **Stopped Servers**: Inactive server count
- **SSL Enabled**: Servers with SSL active
- **Icon Indicators**: Visual representation
- **Color Coding**: Status-appropriate colors

### 🗂️ Data Management

#### SQLite Database
- **Persistent Storage**: All data saved to disk
- **Fast Queries**: Optimized indexes
- **Automatic Creation**: No manual setup
- **Migration System**: Schema versioning
- **Relationships**: Proper foreign keys

#### API Architecture
- **RESTful Design**: Standard HTTP methods
- **JSON Payloads**: Easy to parse and debug
- **Error Handling**: Descriptive error messages
- **CORS Enabled**: Cross-origin requests allowed
- **File Upload Support**: Multipart form data

### 🎯 Navigation & UX

#### Top Navigation Bar
- **Brand Logo**: Icon + gradient text
- **Active Indicators**: Current page highlighted
- **Smooth Animations**: Slide-down effect on load
- **Responsive**: Collapses on mobile
- **Quick Access**: All sections one click away

#### Page Links
- **Dashboard**: Home with system overview
- **Servers**: Server management interface
- **Certificates**: SSL/TLS certificate management

#### Theme Controls
- **Color Palette Icon**: Access accessibility options
- **Dropdown Menu**: All colorblind modes
- **Sun/Moon Toggle**: Switch light/dark theme
- **Persistent State**: Remembers preferences
- **Smooth Transitions**: No jarring changes

### ⚡ Performance Features

#### Optimized Build
- **Code Splitting**: Load only what's needed
- **Tree Shaking**: Remove unused code
- **Minification**: Compressed JavaScript
- **Asset Optimization**: Compressed images and fonts
- **Lazy Loading**: Components load on demand

#### Caching Strategy
- **LocalStorage**: Theme preferences
- **API Response Caching**: Reduced server calls
- **Static Asset Caching**: Faster page loads

#### Efficient Rendering
- **React 18**: Concurrent features
- **Memo Optimization**: Prevent unnecessary re-renders
- **Virtual DOM**: Minimal DOM updates
- **Framer Motion**: GPU-accelerated animations

### 🔒 Security Features

#### Backend Security
- **SQL Injection Prevention**: Parameterized queries with SQLx
- **File Upload Validation**: Type and size checks
- **Path Traversal Prevention**: Sanitized file paths
- **CORS Configuration**: Controlled access
- **Error Sanitization**: No sensitive data in errors

#### Frontend Security
- **XSS Prevention**: React's built-in protection
- **Content Security Policy**: Strict CSP headers
- **HTTPS Enforcement**: Redirect option available
- **Input Validation**: Client-side checks
- **Type Safety**: TypeScript throughout

### 🛠️ Developer Experience

#### TypeScript Support
- **Full Type Coverage**: All components typed
- **Type Definitions**: Custom types in `types/`
- **IntelliSense**: Better IDE support
- **Compile-Time Checks**: Catch errors early

#### Component Library
- **HeroUI**: Modern React components
- **Iconify**: 150,000+ icons
- **Tailwind**: Utility-first CSS
- **Framer Motion**: Animation library

#### Dev Tools
- **Hot Module Replacement**: Instant updates
- **Source Maps**: Debug original code
- **ESLint**: Code quality enforcement
- **Prettier**: Consistent formatting

## Usage Examples

### Creating a Server
```typescript
// Via API
await serverAPI.create({
  name: "My API",
  domain: "api.example.com",
  host: "localhost",
  port: 8080,
  ssl_enabled: true,
  redirect_to_https: true
});
```

### Uploading a Binary
```typescript
// Via file input
const file = event.target.files[0];
await serverAPI.uploadBinary(serverId, file);
```

### Adding a Certificate
```typescript
// Let's Encrypt
await certificateAPI.create({
  name: "My SSL Cert",
  domain: "example.com",
  is_letsencrypt: true
});

// Custom Certificate
const cert = await certificateAPI.create({
  name: "Custom Cert",
  domain: "example.com",
  is_letsencrypt: false
});
await certificateAPI.uploadCertificate(
  cert.id,
  certFile,
  keyFile
);
```

### Fetching Metrics
```typescript
// System stats
const stats = await metricsAPI.getSystemStats();

// Server metrics
const metrics = await metricsAPI.getServerMetrics(serverId);

// Historical data
const history = await metricsAPI.getServerMetricsHistory(serverId);
```

## Future Enhancements

Potential features for future versions:

- **WebSocket Support**: Real-time metrics without polling
- **User Authentication**: Multi-user support with roles
- **Logs Viewer**: View server logs in dashboard
- **Backup/Restore**: Export and import configurations
- **Metrics Graphs**: Historical charts with Chart.js
- **Alert System**: Notifications for server issues
- **Batch Operations**: Manage multiple servers at once
- **API Documentation**: Built-in Swagger UI
- **Container Support**: Docker/Podman integration
- **Load Balancing**: Multiple backend support

## Conclusion

The Minipx Dashboard provides a comprehensive, modern interface for managing reverse proxy servers with a focus on usability, accessibility, and performance. Every feature is designed with user experience in mind, from smooth animations to colorblind-friendly themes to intuitive controls.
