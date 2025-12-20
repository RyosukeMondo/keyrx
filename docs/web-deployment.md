# KeyRx Web Deployment Guide

Complete guide for deploying KeyRx as a web application with backend server.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Prerequisites](#prerequisites)
3. [Backend Server Setup](#backend-server-setup)
4. [Frontend Web Build](#frontend-web-build)
5. [Deployment Options](#deployment-options)
6. [Environment Configuration](#environment-configuration)
7. [Production Considerations](#production-considerations)
8. [Troubleshooting](#troubleshooting)

---

## Architecture Overview

KeyRx's web platform support extends the desktop-only architecture to run in browsers while maintaining the same business logic and feature set.

### Multi-Platform Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Flutter UI Layer                        │
│              (Same codebase for Web + Desktop)               │
└───────────────────────┬─────────────────────────────────────┘
                        │
        ┌───────────────┴───────────────┐
        │                               │
┌───────▼────────┐            ┌─────────▼────────┐
│  Desktop Mode  │            │    Web Mode      │
│   (kIsWeb=F)   │            │   (kIsWeb=T)     │
└───────┬────────┘            └─────────┬────────┘
        │                               │
┌───────▼────────┐            ┌─────────▼────────┐
│  FFI Services  │            │  HTTP Services   │
│  (Direct Rust) │            │  (REST + WS)     │
└───────┬────────┘            └─────────┬────────┘
        │                               │
┌───────▼────────┐            ┌─────────▼────────┐
│  Rust Core     │            │  Backend Server  │
│  (core_logic)  │            │  (Dart+Shelf)    │
└────────────────┘            └─────────┬────────┘
                                        │
                              ┌─────────▼────────┐
                              │  Rust Core       │
                              │  (core_logic)    │
                              └──────────────────┘
```

### Key Differences: Web vs Desktop

| Aspect | Desktop | Web |
|--------|---------|-----|
| **Transport** | FFI (direct memory) | HTTP REST + WebSocket |
| **Latency** | ~0.1ms (in-process) | ~5-10ms (network) |
| **State** | Rust owns state | Server owns state |
| **Real-time** | FFI callbacks | WebSocket events |
| **Dependencies** | core_logic FFI | keyrx_server API |

### SSOT Compliance

Both platforms share:
- **Same business logic** in `packages/core_logic/lib/services/*.dart`
- **Same UI components** in `ui/lib/`
- **Same data models** in `packages/core_logic/lib/models/`

Transport layer differences are isolated via conditional exports:
```dart
export 'services/hardware_profile_service_ffi.dart'
  if (dart.library.html) 'services/hardware_profile_service_web.dart';
```

---

## Prerequisites

### Required Tools

1. **Dart SDK** (≥3.0.0)
   ```bash
   dart --version
   ```

2. **Flutter SDK** (≥3.16.0)
   ```bash
   flutter --version
   flutter config --enable-web
   ```

3. **Rust toolchain** (for core_logic compilation)
   ```bash
   rustc --version
   cargo --version
   ```

### System Requirements

- **Development**: 4GB RAM, 2GB disk space
- **Production Server**: 512MB RAM, 100MB disk (minimal load)
- **Browser Support**: Chrome 90+, Firefox 88+, Safari 14+, Edge 90+

---

## Backend Server Setup

The backend server (`apps/keyrx_server`) wraps core_logic services with a REST API and WebSocket support.

### Step 1: Install Dependencies

```bash
cd apps/keyrx_server
dart pub get
```

### Step 2: Development Mode

Run the server locally for development:

```bash
# Default: localhost:8080
dart run bin/server.dart

# Custom port
dart run bin/server.dart --port 3000

# Custom host (bind to all interfaces)
dart run bin/server.dart --host 0.0.0.0 --port 8080

# Show all options
dart run bin/server.dart --help
```

Server will start and display:
```
KeyRx Backend Server
Listening on http://localhost:8080
Press Ctrl+C to stop
```

### Step 3: Verify Backend

Test the API endpoints:

```bash
# Health check
curl http://localhost:8080/api/devices

# WebSocket test (requires websocat or similar)
websocat ws://localhost:8080/ws
```

### Step 4: Production Build

Compile to standalone executable:

```bash
dart compile exe bin/server.dart -o keyrx-server

# Run compiled binary
./keyrx-server --host 0.0.0.0 --port 8080
```

### API Endpoints Reference

See [apps/keyrx_server/README.md](../apps/keyrx_server/README.md) for complete API documentation.

Key endpoint groups:
- `/api/hardware/profiles` - Hardware profile management
- `/api/keymaps` - Keymap configuration
- `/api/layouts` - Virtual layout definitions
- `/api/devices` - Device discovery and management
- `/api/runtime` - Runtime slot configuration
- `/ws` - WebSocket for real-time events

---

## Frontend Web Build

### Step 1: Environment Configuration

Create environment file for backend URL:

```bash
cd ui
cp web/.env.example web/.env
```

Edit `web/.env`:
```bash
# Backend API server URL
KEYRX_BACKEND_URL=http://localhost:8080

# Base URL path (for subdirectory hosting)
FLUTTER_BASE_HREF=/
```

### Step 2: Development Mode

Run Flutter web with hot reload:

```bash
cd ui
flutter run -d chrome --dart-define=BACKEND_URL=http://localhost:8080
```

### Step 3: Production Build

Build optimized web application:

```bash
cd ui

# Using build script (recommended)
./scripts/build_web.sh

# Or manually
flutter build web \
  --release \
  --base-href=/ \
  --dart-define=BACKEND_URL=http://your-backend-url:8080
```

Build output: `ui/build/web/`

### Step 4: Test Locally

Serve the built files:

```bash
cd ui/build/web
python3 -m http.server 8000
```

Open http://localhost:8000 in browser.

---

## Deployment Options

### Option 1: Single Server (Development/Internal)

Host both frontend and backend on one machine.

```bash
# Terminal 1: Backend
cd apps/keyrx_server
dart run bin/server.dart --host 0.0.0.0 --port 8080

# Terminal 2: Frontend
cd ui/build/web
python3 -m http.server 8000
```

Access: http://your-server:8000

### Option 2: Nginx Reverse Proxy (Production)

Serve static frontend via Nginx, proxy API to backend.

**Nginx configuration** (`/etc/nginx/sites-available/keyrx`):

```nginx
server {
    listen 80;
    server_name keyrx.example.com;

    # Frontend static files
    location / {
        root /var/www/keyrx/web;
        try_files $uri $uri/ /index.html;

        # Cache static assets
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }

    # Backend API proxy
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_cache_bypass $http_upgrade;
    }

    # WebSocket proxy
    location /ws {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400;
    }
}
```

Enable and restart:
```bash
sudo ln -s /etc/nginx/sites-available/keyrx /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

### Option 3: Docker Deployment

**Dockerfile for Backend**:

```dockerfile
FROM dart:stable AS build

WORKDIR /app
COPY apps/keyrx_server/pubspec.* ./
RUN dart pub get

COPY apps/keyrx_server/ ./
RUN dart compile exe bin/server.dart -o server

FROM debian:bullseye-slim
COPY --from=build /app/server /app/server
EXPOSE 8080
CMD ["/app/server", "--host", "0.0.0.0", "--port", "8080"]
```

**docker-compose.yml**:

```yaml
version: '3.8'

services:
  backend:
    build:
      context: .
      dockerfile: Dockerfile.backend
    ports:
      - "8080:8080"
    restart: unless-stopped

  frontend:
    image: nginx:alpine
    volumes:
      - ./ui/build/web:/usr/share/nginx/html:ro
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    ports:
      - "80:80"
    depends_on:
      - backend
    restart: unless-stopped
```

Run:
```bash
docker-compose up -d
```

### Option 4: Systemd Service (Linux)

Create `/etc/systemd/system/keyrx-server.service`:

```ini
[Unit]
Description=KeyRx Backend Server
After=network.target

[Service]
Type=simple
User=keyrx
WorkingDirectory=/opt/keyrx/server
ExecStart=/opt/keyrx/server/keyrx-server --host 0.0.0.0 --port 8080
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable keyrx-server
sudo systemctl start keyrx-server
sudo systemctl status keyrx-server
```

---

## Environment Configuration

### Backend Environment Variables

The server reads from:
1. Command-line arguments (highest priority)
2. Environment variables
3. Default values

**Environment variables**:
```bash
# Server host (default: localhost)
KEYRX_HOST=0.0.0.0

# Server port (default: 8080)
KEYRX_PORT=8080

# Log level (default: INFO)
LOG_LEVEL=DEBUG

# CORS allowed origins (comma-separated)
CORS_ORIGINS=http://localhost:8000,https://keyrx.example.com
```

### Frontend Environment Configuration

Configure at build time via `ui/web/.env`:

```bash
# Required: Backend API URL
KEYRX_BACKEND_URL=http://localhost:8080

# Optional: Subdirectory hosting
# If hosting at https://example.com/keyrx/
FLUTTER_BASE_HREF=/keyrx/
```

Access in Dart:
```dart
const backendUrl = String.fromEnvironment('BACKEND_URL',
  defaultValue: 'http://localhost:8080');
```

### Production Environment Example

**Backend** (systemd environment file `/etc/keyrx/server.env`):
```bash
KEYRX_HOST=127.0.0.1
KEYRX_PORT=8080
LOG_LEVEL=INFO
CORS_ORIGINS=https://keyrx.example.com
```

**Frontend build**:
```bash
cd ui
KEYRX_BACKEND_URL=https://keyrx.example.com \
FLUTTER_BASE_HREF=/ \
./scripts/build_web.sh
```

---

## Production Considerations

### Security

1. **HTTPS**: Always use HTTPS in production
   ```bash
   # Using certbot for Let's Encrypt
   sudo certbot --nginx -d keyrx.example.com
   ```

2. **CORS**: Restrict allowed origins
   ```bash
   CORS_ORIGINS=https://keyrx.example.com
   ```

3. **Firewall**: Limit backend access
   ```bash
   # Backend only accessible from localhost
   KEYRX_HOST=127.0.0.1
   ```

4. **Authentication**: Implement auth layer (future enhancement)

### Performance

1. **Gzip Compression** (Nginx):
   ```nginx
   gzip on;
   gzip_types text/plain text/css application/json application/javascript;
   gzip_min_length 1000;
   ```

2. **HTTP/2** (Nginx):
   ```nginx
   listen 443 ssl http2;
   ```

3. **CDN**: Host static assets on CDN for global reach

4. **Connection Pooling**: Backend handles concurrent WebSocket connections efficiently (tested with 100+ clients)

### Monitoring

1. **Server Logs**: Monitor backend logs
   ```bash
   journalctl -u keyrx-server -f
   ```

2. **Health Check Endpoint**: Implement `/health` endpoint
   ```bash
   curl http://localhost:8080/api/devices
   ```

3. **WebSocket Monitoring**: Track connected clients
   ```dart
   // Backend logs WebSocket connections/disconnections
   ```

4. **Performance Metrics**: See [docs/metrics.md](./metrics.md)

### Backup & Recovery

1. **Data Location**: Core logic state (profiles, keymaps, layouts)
   - Desktop: Local filesystem
   - Web: Server-side storage

2. **Backup Strategy**:
   ```bash
   # Backup server data directory
   tar -czf keyrx-backup-$(date +%Y%m%d).tar.gz /opt/keyrx/data
   ```

3. **Recovery**: Restore data directory and restart server

### Scaling

Current architecture supports:
- **Vertical Scaling**: Increase server resources (single-server sufficient for most use cases)
- **Horizontal Scaling**: Future enhancement - requires session management and load balancing

Typical load:
- 100 concurrent users: 512MB RAM, 1 CPU core
- 1000 concurrent users: 2GB RAM, 2 CPU cores

---

## Troubleshooting

### Backend Issues

**Problem**: Server won't start

```bash
# Check port availability
sudo lsof -i :8080

# Check logs
dart run bin/server.dart --log-level DEBUG

# Verify dependencies
dart pub get
cd ../../packages/core_logic && dart pub get
```

**Problem**: CORS errors in browser console

```
Access to fetch at 'http://localhost:8080/api/devices' from origin
'http://localhost:8000' has been blocked by CORS policy
```

Solution: Configure CORS origins
```bash
CORS_ORIGINS=http://localhost:8000 dart run bin/server.dart
```

**Problem**: WebSocket connection fails

Check browser console for errors:
```javascript
// Should see successful connection
WebSocket connection to 'ws://localhost:8080/ws' failed
```

Solution:
1. Verify WebSocket endpoint: `ws://your-server:8080/ws`
2. Check firewall rules
3. Verify Nginx WebSocket proxy configuration (if using)

### Frontend Issues

**Problem**: White screen after loading

Check browser console:
```
Failed to load resource: net::ERR_CONNECTION_REFUSED
```

Solution: Verify backend URL
```bash
# Rebuild with correct backend URL
cd ui
KEYRX_BACKEND_URL=http://localhost:8080 ./scripts/build_web.sh
```

**Problem**: Build fails with FFI errors

```
Error: Not found: 'dart:ffi'
```

Solution: FFI is stubbed for web - ensure conditional imports are correct:
```dart
// packages/core_logic/lib/core_logic.dart
export 'ffi/bridge_ffi.dart'
  if (dart.library.html) 'ffi/bridge_stub.dart';
```

**Problem**: Real-time updates not working

1. Check WebSocket connection in browser DevTools > Network > WS
2. Verify backend WebSocket handler is running
3. Check for WebSocket proxy issues (if using Nginx)

### Performance Issues

**Problem**: Slow API responses

```bash
# Run backend benchmarks
cd packages/core_logic
dart test test/benchmarks/platform_performance_test.dart
```

Expected latency:
- FFI (desktop): <1ms
- HTTP (web): <20ms (within 20% of desktop)

**Problem**: High memory usage

Monitor backend:
```bash
ps aux | grep keyrx-server
```

Solution: Restart server periodically (if long-running sessions)

### Browser Compatibility

**Problem**: Features not working in Safari

Check browser console for errors. Known issues:
- WebSocket support: Verify Safari 14+
- ES6 features: Verify modern browser version

Solution: Display browser compatibility warning for unsupported browsers

### Deployment Issues

**Problem**: Nginx 502 Bad Gateway

```bash
# Check backend is running
curl http://localhost:8080/api/devices

# Check Nginx error log
sudo tail -f /var/log/nginx/error.log

# Verify proxy configuration
sudo nginx -t
```

**Problem**: WebSocket disconnects after 60 seconds

Solution: Increase Nginx proxy timeout
```nginx
location /ws {
    proxy_read_timeout 86400;  # 24 hours
}
```

---

## Next Steps

1. **Production Deployment**: Follow [Option 2: Nginx Reverse Proxy](#option-2-nginx-reverse-proxy-production)
2. **Monitoring Setup**: Implement health checks and logging
3. **Security Hardening**: Enable HTTPS, restrict CORS, add authentication
4. **Performance Tuning**: Enable gzip, HTTP/2, CDN
5. **Backup Strategy**: Automate data backups

---

## Additional Resources

- [Backend API Documentation](../apps/keyrx_server/README.md)
- [Architecture Overview](./ARCHITECTURE.md)
- [Performance Metrics](./metrics.md)
- [Development Guide](./AUTONOMOUS-DEV-GUIDE.md)
- [Flutter Web Documentation](https://docs.flutter.dev/platform-integration/web)
- [Shelf Server Documentation](https://pub.dev/packages/shelf)

---

## Support

For issues or questions:
1. Check [Troubleshooting](#troubleshooting) section
2. Review backend logs: `journalctl -u keyrx-server -f`
3. Check browser console for frontend errors
4. Verify environment configuration

**Key Debugging Commands**:
```bash
# Test backend connectivity
curl -v http://localhost:8080/api/devices

# Test WebSocket
websocat ws://localhost:8080/ws

# Check frontend build
cd ui/build/web && python3 -m http.server 8000

# View server logs
dart run bin/server.dart --log-level DEBUG
```
