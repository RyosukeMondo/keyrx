#!/bin/bash
# HTTPS Development Server for Flutter Web
# Generates self-signed certificate and starts HTTPS server for service worker testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build/web"
CERT_DIR="$PROJECT_ROOT/.dev-certs"
PORT="${PORT:-4443}"
BIND_IP="${BIND_IP:-0.0.0.0}"

# Stop any existing servers on this port
echo "🧹 Checking for existing servers on port $PORT..."
pkill -f "python.*$PORT" 2>/dev/null || true
sleep 1

echo "🔐 KeyRx HTTPS Development Server"
echo "=================================="
echo ""

# Create cert directory if it doesn't exist
mkdir -p "$CERT_DIR"

# Generate self-signed certificate if it doesn't exist
if [ ! -f "$CERT_DIR/cert.pem" ] || [ ! -f "$CERT_DIR/key.pem" ]; then
    echo "📜 Generating self-signed SSL certificate..."
    openssl req -x509 -newkey rsa:4096 -nodes \
        -keyout "$CERT_DIR/key.pem" \
        -out "$CERT_DIR/cert.pem" \
        -days 365 \
        -subj "/C=US/ST=Dev/L=Local/O=KeyRx/CN=localhost" \
        -addext "subjectAltName=DNS:localhost,IP:127.0.0.1,IP:192.168.11.24,IP:192.168.11.53" \
        2>/dev/null

    echo "✅ Certificate generated"
    echo ""
fi

# Check if build exists
if [ ! -d "$BUILD_DIR" ]; then
    echo "❌ Build directory not found: $BUILD_DIR"
    echo "Run 'flutter build web' first"
    exit 1
fi

echo "📁 Serving: $BUILD_DIR"
echo "🌐 Address: https://$BIND_IP:$PORT"
echo "🔗 Access via:"
echo "   - https://localhost:$PORT"
echo "   - https://192.168.11.24:$PORT"
echo "   - https://192.168.11.53:$PORT"
echo ""
echo "⚠️  Browser will show security warning (self-signed cert)"
echo "   Click 'Advanced' → 'Proceed to localhost (unsafe)'"
echo ""
echo "🛑 Press Ctrl+C to stop"
echo ""

# Start HTTPS server using Python
cd "$BUILD_DIR"

# Python 3 HTTPS server
python3 - <<EOF
import http.server
import ssl
import os

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Add headers for service worker support
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        super().end_headers()

    def log_message(self, format, *args):
        # Custom logging
        print(f"[{self.address_string()}] {format % args}")

server_address = ('$BIND_IP', $PORT)
httpd = http.server.HTTPServer(server_address, MyHTTPRequestHandler)

context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain('$CERT_DIR/cert.pem', '$CERT_DIR/key.pem')

httpd.socket = context.wrap_socket(httpd.socket, server_side=True)

print(f"🚀 HTTPS Server started on https://$BIND_IP:$PORT")
print(f"📂 Serving: $(pwd)")
print("")

try:
    httpd.serve_forever()
except KeyboardInterrupt:
    print("")
    print("🛑 Server stopped")
    httpd.shutdown()
EOF
