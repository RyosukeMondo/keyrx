# Development Server Scripts

Scripts for serving Flutter web builds during development.

## Quick Start

### HTTP Server (Simple)
```bash
./scripts/start-http-dev-server.sh
```
- **Port:** 4200 (default)
- **URL:** http://localhost:4200
- **Use for:** Quick testing, debugging
- **Note:** Service workers won't work (requires HTTPS)

### HTTPS Server (Service Workers)
```bash
./scripts/start-https-dev-server.sh
```
- **Port:** 4443 (default)
- **URL:** https://localhost:4443
- **Use for:** Service worker testing, PWA features
- **Note:** Browser will show security warning (self-signed cert)

## Custom Port

```bash
PORT=8080 ./scripts/start-http-dev-server.sh
PORT=8443 ./scripts/start-https-dev-server.sh
```

## Custom Bind IP

```bash
BIND_IP=127.0.0.1 ./scripts/start-http-dev-server.sh  # Localhost only
BIND_IP=0.0.0.0 ./scripts/start-https-dev-server.sh   # All interfaces
```

## HTTPS Certificate

On first run, HTTPS server generates a self-signed certificate:
- Location: `ui/.dev-certs/`
- Valid for: 365 days
- Includes: localhost, 127.0.0.1, 192.168.11.24, 192.168.11.53

### Accepting Self-Signed Certificate

**Chrome/Edge:**
1. Navigate to `https://localhost:4443`
2. Click "Advanced"
3. Click "Proceed to localhost (unsafe)"

**Firefox:**
1. Navigate to `https://localhost:4443`
2. Click "Advanced"
3. Click "Accept the Risk and Continue"

### Regenerating Certificate

```bash
rm -rf ui/.dev-certs
./scripts/start-https-dev-server.sh  # Will generate new cert
```

## Before Running

Build the web app first:
```bash
cd ui
flutter build web --release
```

## Server Status

Check what's running:
```bash
ss -tlnp | grep -E "4200|4443"
```

Stop servers:
```bash
pkill -f "python.*http.server"
```

## Troubleshooting

### Port Already in Use
```bash
# Find what's using the port
lsof -i :4200
lsof -i :4443

# Kill the process
pkill -f "python.*4200"
pkill -f "python.*4443"
```

### Service Worker Not Registering
- ✅ Use HTTPS server (`start-https-dev-server.sh`)
- ✅ Accept self-signed certificate in browser
- ✅ Check browser console for errors
- ✅ Verify service worker is enabled in browser settings

### Build Not Found
```bash
cd ui
flutter build web --release
```

## Backend Integration

Both servers serve only the frontend. Backend must run separately:

```bash
# In another terminal
cd apps/keyrx_server
dart run bin/server.dart
```

Default backend: `http://localhost:8888`

Configure in `ui/lib/state/providers_setup.dart`:
```dart
backendUrl: kIsWeb ? 'http://192.168.11.24:8888' : null,
```

## Production Deployment

These scripts are for **development only**. For production:

1. Build optimized release:
   ```bash
   flutter build web --release
   ```

2. Use production web server:
   - Nginx
   - Apache
   - Caddy
   - Cloud hosting (Firebase, Netlify, Vercel)

3. Configure proper SSL/TLS with real certificate

## Network Access

### Local Network Access
Scripts bind to `0.0.0.0` by default, making the server accessible on your network:

- **Your machine:** https://localhost:4443
- **Other devices:** https://192.168.11.24:4443
- **WiFi devices:** https://192.168.11.53:4443

### Firewall Configuration

If other devices can't connect:

```bash
# Check firewall status
sudo ufw status

# Allow port (if firewall is active)
sudo ufw allow 4200/tcp
sudo ufw allow 4443/tcp
```

## Security Notes

⚠️ **Development Only**
- Self-signed certificates provide encryption but NO identity verification
- Anyone can intercept/modify traffic on first connection
- Do NOT use for sensitive data
- Do NOT expose to public internet

✅ **Safe for:**
- Local development
- Testing on trusted local network
- PWA/service worker development
- Mobile device testing on local WiFi

❌ **NOT safe for:**
- Production deployments
- Public internet exposure
- Handling real user data
- Compliance requirements (PCI, HIPAA, etc.)
